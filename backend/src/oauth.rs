use std::sync::Arc;
use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{auth::token::new_refresh_token, error::{AppError, AppResult}, state::AppState};

#[derive(Deserialize)]
struct OAuthRequest { provider: String, provider_user_id: String, email: String, display_name: Option<String> }

#[derive(Serialize)]
struct OAuthResponse { access_token: String, refresh_token: String, user_id: Uuid, provider: String, is_new: bool }

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/auth/oauth")
        .route("/google", web::post().to(oauth_login))
        .route("/discord", web::post().to(oauth_login))
    );
}

async fn oauth_login(state: web::Data<Arc<AppState>>, req: HttpRequest, body: web::Json<OAuthRequest>) -> AppResult<HttpResponse> {
    let provider = body.provider.to_lowercase();
    if !["google","discord"].contains(&provider.as_str()) { return Err(AppError::Validation("provider deve ser google ou discord".into())); }
    if body.provider_user_id.len() < 3 { return Err(AppError::Validation("provider_user_id inválido".into())); }
    let email = body.email.trim().to_lowercase();
    if !email.contains('@') { return Err(AppError::Validation("email inválido".into())); }
    let display_name = body.display_name.clone().unwrap_or_else(|| email.split('@').next().unwrap_or("Hero").to_string());
    let mut tx = state.db.begin().await?;
    // Busca por oauth account existente
    let existing: Option<Uuid> = sqlx::query_scalar("SELECT user_id FROM oauth_accounts WHERE provider=$1 AND provider_user_id=$2")
        .bind(&provider).bind(&body.provider_user_id).fetch_optional(&mut *tx).await?;
    let user_id = if let Some(uid) = existing {
        uid
    } else {
        // Verifica se email já existe -> linka
        let by_email: Option<Uuid> = sqlx::query_scalar("SELECT id FROM users WHERE lower(email)=$1").bind(&email).fetch_optional(&mut *tx).await?;
        if let Some(uid) = by_email {
            sqlx::query("INSERT INTO oauth_accounts (user_id, provider, provider_user_id, email) VALUES ($1,$2,$3,$4) ON CONFLICT DO NOTHING")
                .bind(uid).bind(&provider).bind(&body.provider_user_id).bind(&email).execute(&mut *tx).await?;
            uid
        } else {
            // Cria novo usuário com senha placeholder (OAuth não usa senha)
            let uid = Uuid::new_v4();
            let placeholder_hash = "$argon2id$v=19$m=19456,t=2,p=1$placeholder$placeholder";
            sqlx::query("INSERT INTO users (id, email, password_hash, display_name) VALUES ($1,$2,$3,$4)")
                .bind(uid).bind(&email).bind(placeholder_hash).bind(display_name.trim()).execute(&mut *tx).await?;
            let cid = Uuid::new_v4();
            sqlx::query("INSERT INTO characters (id,user_id,name,gender,class,subclass,is_leader) VALUES ($1,$2,$3,'male'::character_gender,'commander','emperor',true)")
                .bind(cid).bind(uid).bind(display_name.trim()).execute(&mut *tx).await?;
            sqlx::query("INSERT INTO character_stats (character_id) VALUES ($1)").bind(cid).execute(&mut *tx).await?;
            sqlx::query("INSERT INTO character_base_stats (character_id) VALUES ($1)").bind(cid).execute(&mut *tx).await?;
            crate::inventory::stats::recalculate(&mut tx, uid, cid).await?;
            let sid = Uuid::new_v4();
            sqlx::query("INSERT INTO squads (id,user_id,name) VALUES ($1,$2,'Principal')").bind(sid).bind(uid).execute(&mut *tx).await?;
            sqlx::query("INSERT INTO squad_slots (squad_id,slot,character_id) VALUES ($1,1,$2)").bind(sid).bind(cid).execute(&mut *tx).await?;
            sqlx::query("INSERT INTO oauth_accounts (user_id, provider, provider_user_id, email) VALUES ($1,$2,$3,$4)")
                .bind(uid).bind(&provider).bind(&body.provider_user_id).bind(&email).execute(&mut *tx).await?;
            uid
        }
    };
    // Cria sessão
    let session_id = Uuid::new_v4();
    let refresh_token = new_refresh_token();
    let expires_at = chrono::Utc::now() + chrono::Duration::days(state.config.refresh_token_days);
    let ua = req.headers().get("User-Agent").and_then(|v| v.to_str().ok()).unwrap_or("");
    let ip = req.connection_info().realip_remote_addr().unwrap_or("").to_owned();
    let hash = crate::auth::token::token_hash(&refresh_token);
    sqlx::query("INSERT INTO refresh_sessions (id,user_id,token_hash,expires_at,user_agent,ip_address) VALUES ($1,$2,$3,$4,$5,$6)")
        .bind(session_id).bind(user_id).bind(hash).bind(expires_at).bind(ua).bind(ip).execute(&mut *tx).await?;
    let is_new = existing.is_none();
    sqlx::query("INSERT INTO audit_logs (actor_user_id,action,metadata) VALUES ($1,'OAUTH_LOGIN',$2)").bind(user_id).bind(serde_json::json!({"provider":provider,"is_new":is_new})).execute(&mut *tx).await?;
    tx.commit().await?;
    let access_token = state.tokens.access_token(user_id, session_id)?;
    Ok(HttpResponse::Ok().json(OAuthResponse{ access_token, refresh_token, user_id, provider, is_new }))
}
