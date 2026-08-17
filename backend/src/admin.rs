use std::sync::Arc;
use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{auth::middleware::AuthenticatedUser, error::{AppError, AppResult}, state::AppState};

#[derive(Deserialize)]
struct BanRequest { user_id: Uuid, reason: String, duration_hours: Option<i64> }

#[derive(Serialize)]
struct AdminUserView { id: Uuid, email: String, display_name: String, status: String, vip_level: i16, is_admin: bool, is_gm: bool }

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/admin")
        .route("/users", web::get().to(list_users))
        .route("/ban", web::post().to(ban))
        .route("/unban", web::post().to(unban))
        .route("/logs", web::get().to(logs))
        .route("/metrics", web::get().to(metrics))
        .route("/grant-admin", web::post().to(grant_admin))
    );
}

async fn require_admin(state: &AppState, user: &AuthenticatedUser) -> AppResult<()> {
    let (is_admin, is_gm): (bool,bool) = sqlx::query_as("SELECT is_admin, is_gm FROM users WHERE id=$1").bind(user.user_id).fetch_one(&state.db).await?;
    if is_admin || is_gm { Ok(()) } else { Err(AppError::Unauthorized) }
}

async fn list_users(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, req: HttpRequest) -> AppResult<HttpResponse> {
    require_admin(&state, &user).await?;
    let users: Vec<AdminUserView> = sqlx::query_as("SELECT id, email, display_name, status::text as status, vip_level, is_admin, is_gm FROM users ORDER BY created_at DESC LIMIT 50")
        .fetch_all(&state.db).await?;
    Ok(HttpResponse::Ok().json(users))
}

async fn ban(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, body: web::Json<BanRequest>) -> AppResult<HttpResponse> {
    require_admin(&state, &user).await?;
    sqlx::query("UPDATE users SET status='banned' WHERE id=$1").bind(body.user_id).execute(&state.db).await?;
    sqlx::query("INSERT INTO admin_logs (admin_user_id, action, target_user_id, metadata) VALUES ($1,'BAN',$2,$3)")
        .bind(user.user_id).bind(body.user_id).bind(serde_json::json!({"reason":body.reason,"duration_hours":body.duration_hours})).execute(&state.db).await?;
    // Revoga sessões
    sqlx::query("UPDATE refresh_sessions SET revoked_at=now() WHERE user_id=$1 AND revoked_at IS NULL").bind(body.user_id).execute(&state.db).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"banned":body.user_id})))
}

async fn unban(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, body: web::Json<BanRequest>) -> AppResult<HttpResponse> {
    require_admin(&state, &user).await?;
    sqlx::query("UPDATE users SET status='active' WHERE id=$1").bind(body.user_id).execute(&state.db).await?;
    sqlx::query("INSERT INTO admin_logs (admin_user_id, action, target_user_id, metadata) VALUES ($1,'UNBAN',$2,$3)")
        .bind(user.user_id).bind(body.user_id).bind(serde_json::json!({})).execute(&state.db).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"unbanned":body.user_id})))
}

async fn logs(state: web::Data<Arc<AppState>>, user: AuthenticatedUser) -> AppResult<HttpResponse> {
    require_admin(&state, &user).await?;
    let rows: Vec<(i64,Option<Uuid>,String,serde_json::Value,chrono::DateTime<chrono::Utc>)> = sqlx::query_as("SELECT id, admin_user_id, action, metadata, created_at FROM admin_logs ORDER BY created_at DESC LIMIT 50")
        .fetch_all(&state.db).await?;
    Ok(HttpResponse::Ok().json(rows.into_iter().map(|(id,admin,action,meta,at)| serde_json::json!({"id":id,"admin":admin,"action":action,"meta":meta,"at":at})).collect::<Vec<_>>()))
}

async fn metrics(state: web::Data<Arc<AppState>>, user: AuthenticatedUser) -> AppResult<HttpResponse> {
    require_admin(&state, &user).await?;
    let counters: Vec<(String,i64)> = sqlx::query_as("SELECT name, value FROM metrics_counters ORDER BY name").fetch_all(&state.db).await?;
    let health: String = {
        let mut conn = state.redis.get_multiplexed_async_connection().await.map_err(|_| AppError::Internal("redis".into()))?;
        redis::cmd("PING").query_async(&mut conn).await.unwrap_or("PONG".to_string())
    };
    Ok(HttpResponse::Ok().json(serde_json::json!({"counters":counters.into_iter().map(|(n,v)| serde_json::json!({"name":n,"value":v})).collect::<Vec<_>>(),"redis":health,"uptime":"ok"})))
}

#[derive(Deserialize)]
struct GrantAdmin { user_id: Uuid, is_admin: bool, is_gm: bool }

async fn grant_admin(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, body: web::Json<GrantAdmin>) -> AppResult<HttpResponse> {
    // Primeiro admin pode ser criado via env: ADMIN_EMAIL
    let (self_admin,): (bool,) = sqlx::query_as("SELECT is_admin FROM users WHERE id=$1").bind(user.user_id).fetch_one(&state.db).await?;
    let admin_email = std::env::var("ADMIN_EMAIL").unwrap_or_default();
    let self_email: String = sqlx::query_scalar("SELECT email FROM users WHERE id=$1").bind(user.user_id).fetch_one(&state.db).await?;
    if !self_admin && self_email != admin_email {
        return Err(AppError::Unauthorized);
    }
    sqlx::query("UPDATE users SET is_admin=$2, is_gm=$3 WHERE id=$1").bind(body.user_id).bind(body.is_admin).bind(body.is_gm).execute(&state.db).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"granted":body.user_id,"admin":body.is_admin,"gm":body.is_gm})))
}
