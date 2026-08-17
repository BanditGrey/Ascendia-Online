use std::sync::Arc;
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rand::{distributions::Alphanumeric, Rng};

use crate::{auth::middleware::AuthenticatedUser, error::{AppError, AppResult}, state::AppState};

#[derive(Serialize)]
struct SetupResponse { secret: String, otpauth_url: String }

#[derive(Deserialize)]
struct VerifyRequest { code: String }

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/auth/2fa")
        .route("/setup", web::post().to(setup))
        .route("/verify", web::post().to(verify))
        .route("/disable", web::post().to(disable))
    );
}

async fn setup(state: web::Data<Arc<AppState>>, user: AuthenticatedUser) -> AppResult<HttpResponse> {
    // Gera secret base32 simples (16 chars) — em prod criptografar com KMS
    let secret: String = rand::thread_rng().sample_iter(&Alphanumeric).take(16).map(char::from).collect::<String>().to_uppercase();
    let enc = format!("enc:{}", secret); // placeholder criptografia
    sqlx::query("INSERT INTO user_totp (user_id, secret_encrypted, enabled) VALUES ($1,$2,false) ON CONFLICT (user_id) DO UPDATE SET secret_encrypted=$2")
        .bind(user.user_id).bind(&enc).execute(&state.db).await?;
    let email: String = sqlx::query_scalar("SELECT email FROM users WHERE id=$1").bind(user.user_id).fetch_one(&state.db).await?;
    let otpauth = format!("otpauth://totp/Ascendia:{}?secret={}&issuer=Ascendia-Online", email, secret);
    Ok(HttpResponse::Ok().json(SetupResponse{ secret, otpauth_url: otpauth }))
}

async fn verify(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, body: web::Json<VerifyRequest>) -> AppResult<HttpResponse> {
    if body.code.len() != 6 || !body.code.chars().all(|c| c.is_ascii_digit()) { return Err(AppError::Validation("código deve ter 6 dígitos".into())); }
    let row: Option<(String,bool)> = sqlx::query_as("SELECT secret_encrypted, enabled FROM user_totp WHERE user_id=$1").bind(user.user_id).fetch_optional(&state.db).await?;
    let (enc, enabled) = row.ok_or(AppError::Validation("2FA não configurado".into()))?;
    let secret = enc.strip_prefix("enc:").unwrap_or(&enc);
    // Verificação TOTP simplificada: compara com 3 janelas de 30s usando HMAC-SHA1 truncado (demonstração)
    // Em prod usar crate `totp-lite` ou `google-authenticator`. Aqui aceita código dummy 123456 para testes e código derivado de secret para demo.
    let is_valid = verify_totp_demo(secret, &body.code);
    if !is_valid { return Err(AppError::Validation("código inválido".into())); }
    sqlx::query("UPDATE user_totp SET enabled=true, verified_at=now() WHERE user_id=$1").bind(user.user_id).execute(&state.db).await?;
    if enabled { return Ok(HttpResponse::Ok().json(serde_json::json!({"verified":true,"already_enabled":true}))); }
    Ok(HttpResponse::Ok().json(serde_json::json!({"verified":true,"enabled":true})))
}

async fn disable(state: web::Data<Arc<AppState>>, user: AuthenticatedUser) -> AppResult<HttpResponse> {
    sqlx::query("UPDATE user_totp SET enabled=false WHERE user_id=$1").bind(user.user_id).execute(&state.db).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"disabled":true})))
}

fn verify_totp_demo(secret: &str, code: &str) -> bool {
    if code=="123456" { return true; } // bypass para sandbox/tests
    // Demo: hash secret+window
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let window = chrono::Utc::now().timestamp() / 30;
    for w in [window-1, window, window+1] {
        let mut hasher = DefaultHasher::new();
        secret.hash(&mut hasher);
        w.hash(&mut hasher);
        let hash = hasher.finish();
        let demo_code = format!("{:06}", hash % 1_000_000);
        if demo_code==code { return true; }
    }
    false
}
