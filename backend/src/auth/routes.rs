use std::sync::Arc;

use actix_web::{web, HttpRequest, HttpResponse};
use argon2::{password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString}, Argon2};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Postgres;
use uuid::Uuid;
use validator::Validate;

use crate::{auth::{middleware::AuthenticatedUser, token::{new_refresh_token, token_hash}}, error::{AppError, AppResult}, inventory::stats::recalculate, state::AppState};

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "e-mail inválido"))]
    pub email: String,
    #[validate(length(min = 3, max = 24, message = "nome deve ter entre 3 e 24 caracteres"))]
    pub display_name: String,
    #[validate(length(min = 10, max = 128, message = "senha deve ter no mínimo 10 caracteres"))]
    pub password: String,
    pub gender: Gender,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Gender { Male, Female }

impl Gender { fn as_str(&self) -> &'static str { match self { Self::Male => "male", Self::Female => "female" } } }

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest { pub refresh_token: String }

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: &'static str,
    pub expires_in: i64,
    pub user_id: Uuid,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
            .route("/refresh", web::post().to(refresh))
            .route("/logout", web::post().to(logout)),
    );
}

async fn register(state: web::Data<Arc<AppState>>, body: web::Json<RegisterRequest>, req: HttpRequest) -> AppResult<HttpResponse> {
    body.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let email = body.email.trim().to_lowercase();
    let display_name = body.display_name.trim();
    let password_hash = hash_password(body.password.clone()).await?;
    let mut tx = state.db.begin().await?;
    let user_id = Uuid::new_v4();

    let inserted = sqlx::query("INSERT INTO users (id, email, password_hash, display_name) VALUES ($1,$2,$3,$4)")
        .bind(user_id).bind(email).bind(password_hash).bind(display_name)
        .execute(&mut *tx).await;
    if let Err(error) = inserted {
        if matches!(&error, sqlx::Error::Database(db) if db.is_unique_violation()) {
            return Err(AppError::Conflict("e-mail ou nome já está em uso".into()));
        }
        return Err(error.into());
    }

    let character_id = Uuid::new_v4();
    sqlx::query("INSERT INTO characters (id,user_id,name,gender,class,subclass,is_leader) VALUES ($1,$2,$3,$4::text::character_gender,'commander','emperor',true)")
        .bind(character_id).bind(user_id).bind(display_name).bind(body.gender.as_str())
        .execute(&mut *tx).await?;
    sqlx::query("INSERT INTO character_stats (character_id) VALUES ($1)")
        .bind(character_id).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO character_base_stats (character_id) VALUES ($1)")
        .bind(character_id).execute(&mut *tx).await?;
    recalculate(&mut tx, user_id, character_id).await?;
    let squad_id = Uuid::new_v4();
    sqlx::query("INSERT INTO squads (id,user_id,name) VALUES ($1,$2,'Principal')")
        .bind(squad_id).bind(user_id).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO squad_slots (squad_id,slot,character_id) VALUES ($1,1,$2)")
        .bind(squad_id).bind(character_id).execute(&mut *tx).await?;

    let response = create_session(&state, &mut tx, user_id, &req).await?;
    audit(&mut tx, user_id, "USER_REGISTERED", serde_json::json!({ "character_id": character_id })).await?;
    tx.commit().await?;
    log::info!("usuário registrado user_id={user_id}");
    Ok(HttpResponse::Created().json(response))
}

async fn login(state: web::Data<Arc<AppState>>, body: web::Json<LoginRequest>, req: HttpRequest) -> AppResult<HttpResponse> {
    body.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let row: Option<(Uuid, String)> = sqlx::query_as("SELECT id,password_hash FROM users WHERE email=$1 AND status='active'")
        .bind(body.email.trim().to_lowercase()).fetch_optional(&state.db).await?;
    let (user_id, password_hash) = row.ok_or(AppError::InvalidCredentials)?;
    verify_password(password_hash, body.password.clone()).await?;
    let mut tx = state.db.begin().await?;
    let response = create_session(&state, &mut tx, user_id, &req).await?;
    sqlx::query("UPDATE users SET last_login_at=now() WHERE id=$1").bind(user_id).execute(&mut *tx).await?;
    audit(&mut tx, user_id, "USER_LOGIN", serde_json::json!({})).await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(response))
}

async fn refresh(state: web::Data<Arc<AppState>>, body: web::Json<RefreshRequest>, req: HttpRequest) -> AppResult<HttpResponse> {
    let old_hash = token_hash(&body.refresh_token);
    let mut tx = state.db.begin().await?;
    let row: Option<(Uuid, Uuid)> = sqlx::query_as(
        "SELECT id,user_id FROM refresh_sessions WHERE token_hash=$1 AND revoked_at IS NULL AND expires_at>now() FOR UPDATE"
    ).bind(old_hash).fetch_optional(&mut *tx).await?;
    let (old_session, user_id) = row.ok_or(AppError::Unauthorized)?;
    // Rotação de refresh token: cada token só pode ser utilizado uma vez.
    sqlx::query("UPDATE refresh_sessions SET revoked_at=now() WHERE id=$1")
        .bind(old_session).execute(&mut *tx).await?;
    let response = create_session(&state, &mut tx, user_id, &req).await?;
    audit(&mut tx, user_id, "TOKEN_REFRESHED", serde_json::json!({ "old_session": old_session })).await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(response))
}

async fn logout(state: web::Data<Arc<AppState>>, user: AuthenticatedUser) -> AppResult<HttpResponse> {
    let mut tx = state.db.begin().await?;
    sqlx::query("UPDATE refresh_sessions SET revoked_at=COALESCE(revoked_at,now()) WHERE id=$1 AND user_id=$2")
        .bind(user.session_id).bind(user.user_id).execute(&mut *tx).await?;
    audit(&mut tx, user.user_id, "USER_LOGOUT", serde_json::json!({ "session_id": user.session_id })).await?;
    tx.commit().await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn create_session(state: &AppState, tx: &mut sqlx::Transaction<'_, Postgres>, user_id: Uuid, req: &HttpRequest) -> AppResult<AuthResponse> {
    let session_id = Uuid::new_v4();
    let refresh_token = new_refresh_token();
    let expires_at = Utc::now() + Duration::days(state.config.refresh_token_days);
    let user_agent = req.headers().get("User-Agent").and_then(|v| v.to_str().ok()).unwrap_or("");
    let ip = req.connection_info().realip_remote_addr().unwrap_or("").to_owned();
    sqlx::query("INSERT INTO refresh_sessions (id,user_id,token_hash,expires_at,user_agent,ip_address) VALUES ($1,$2,$3,$4,$5,$6)")
        .bind(session_id).bind(user_id).bind(token_hash(&refresh_token)).bind(expires_at).bind(user_agent).bind(ip)
        .execute(&mut **tx).await?;
    Ok(AuthResponse {
        access_token: state.tokens.access_token(user_id, session_id)?,
        refresh_token,
        token_type: "Bearer",
        expires_in: state.config.access_token_minutes * 60,
        user_id,
    })
}

async fn audit(tx: &mut sqlx::Transaction<'_, Postgres>, actor: Uuid, action: &str, metadata: serde_json::Value) -> AppResult<()> {
    sqlx::query("INSERT INTO audit_logs (actor_user_id,action,metadata) VALUES ($1,$2,$3)")
        .bind(actor).bind(action).bind(metadata).execute(&mut **tx).await?;
    Ok(())
}

async fn hash_password(password: String) -> AppResult<String> {
    actix_web::rt::task::spawn_blocking(move || {
        Argon2::default().hash_password(password.as_bytes(), &SaltString::generate(&mut OsRng))
            .map(|h| h.to_string()).map_err(|e| AppError::Internal(format!("password hash: {e}")))
    }).await.map_err(|e| AppError::Internal(format!("hash task: {e}")))?
}

async fn verify_password(hash: String, password: String) -> AppResult<()> {
    actix_web::rt::task::spawn_blocking(move || {
        let parsed = PasswordHash::new(&hash).map_err(|_| AppError::InvalidCredentials)?;
        Argon2::default().verify_password(password.as_bytes(), &parsed).map_err(|_| AppError::InvalidCredentials)
    }).await.map_err(|e| AppError::Internal(format!("verify task: {e}")))?
}
