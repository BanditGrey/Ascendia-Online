use std::sync::Arc;

use actix_web::{web, HttpResponse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

use crate::{auth::middleware::AuthenticatedUser, error::{AppError, AppResult}, state::AppState};

const GLOBAL_HISTORY_KEY: &str = "chat:global:history:v1";
const GLOBAL_HISTORY_LIMIT: isize = 100;
const RATE_LIMIT_SECONDS: usize = 3;

#[derive(Deserialize, Validate)]
struct SendGlobal {
    #[validate(length(min = 1, max = 280))]
    content: String,
}
#[derive(Deserialize, Validate)]
struct SendWhisper {
    recipient_user_id: Uuid,
    #[validate(length(min = 1, max = 280))]
    content: String,
}
#[derive(Deserialize)]
struct BlockRequest { user_id: Uuid }
#[derive(Deserialize, Validate)]
struct ReportRequest { message_id: Uuid, #[validate(length(min = 3, max = 280))] reason: String }
#[derive(Deserialize)]
struct HistoryQuery { #[serde(default = "default_limit")] limit: i64 }
fn default_limit() -> i64 { 50 }

#[derive(FromRow, Clone, Serialize)]
struct ChatMessage { id: Uuid, sender_user_id: Uuid, sender_name: String, recipient_user_id: Option<Uuid>, channel: String, content: String, created_at: DateTime<Utc> }

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/chat")
        .route("/global", web::get().to(global_history))
        .route("/global", web::post().to(send_global))
        .route("/whisper", web::post().to(send_whisper))
        .route("/blocks", web::post().to(block))
        .route("/blocks/{user_id}", web::delete().to(unblock))
        .route("/reports", web::post().to(report)));
}

async fn send_global(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, body: web::Json<SendGlobal>) -> AppResult<HttpResponse> {
    body.validate().map_err(|error| AppError::Validation(error.to_string()))?;
    let content = sanitize(&body.content)?;
    enforce_rate_limit(&state, user.user_id).await?;
    let message = insert_message(&state, user.user_id, None, "global", content).await?;
    cache_global(&state, &message).await;
    Ok(HttpResponse::Created().json(message))
}

async fn send_whisper(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, body: web::Json<SendWhisper>) -> AppResult<HttpResponse> {
    body.validate().map_err(|error| AppError::Validation(error.to_string()))?;
    if user.user_id == body.recipient_user_id { return Err(AppError::Validation("não é possível enviar whisper para si mesmo".into())); }
    let content = sanitize(&body.content)?;
    enforce_rate_limit(&state, user.user_id).await?;
    let allowed: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users u WHERE u.id=$1 AND u.status='active') AND NOT EXISTS(SELECT 1 FROM user_blocks WHERE user_id=$1 AND blocked_user_id=$2)")
        .bind(body.recipient_user_id).bind(user.user_id).fetch_one(&state.db).await?;
    if !allowed { return Err(AppError::Validation("destinatário indisponível para mensagens".into())); }
    let message = insert_message(&state, user.user_id, Some(body.recipient_user_id), "whisper", content).await?;
    Ok(HttpResponse::Created().json(message))
}

async fn global_history(state: web::Data<Arc<AppState>>, _user: AuthenticatedUser, query: web::Query<HistoryQuery>) -> AppResult<HttpResponse> {
    let limit = query.limit.clamp(1, 100);
    let cached = read_global_cache(&state, limit).await?;
    if !cached.is_empty() { return Ok(HttpResponse::Ok().json(cached)); }
    let messages: Vec<ChatMessage> = sqlx::query_as("SELECT m.id,m.sender_user_id,u.display_name AS sender_name,m.recipient_user_id,m.channel,m.content,m.created_at FROM chat_messages m JOIN users u ON u.id=m.sender_user_id WHERE m.channel='global' ORDER BY m.created_at DESC LIMIT $1")
        .bind(limit).fetch_all(&state.db).await?;
    for message in messages.iter().rev() { cache_global(&state, message).await; }
    Ok(HttpResponse::Ok().json(messages))
}

async fn block(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, body: web::Json<BlockRequest>) -> AppResult<HttpResponse> {
    if user.user_id == body.user_id { return Err(AppError::Validation("não é possível bloquear a si mesmo".into())); }
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id=$1)").bind(body.user_id).fetch_one(&state.db).await?;
    if !exists { return Err(AppError::NotFound); }
    sqlx::query("INSERT INTO user_blocks (user_id,blocked_user_id) VALUES ($1,$2) ON CONFLICT DO NOTHING").bind(user.user_id).bind(body.user_id).execute(&state.db).await?;
    audit(&state, user.user_id, "CHAT_USER_BLOCKED", serde_json::json!({"blocked_user_id":body.user_id})).await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn unblock(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, blocked_user_id: web::Path<Uuid>) -> AppResult<HttpResponse> {
    sqlx::query("DELETE FROM user_blocks WHERE user_id=$1 AND blocked_user_id=$2").bind(user.user_id).bind(*blocked_user_id).execute(&state.db).await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn report(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, body: web::Json<ReportRequest>) -> AppResult<HttpResponse> {
    body.validate().map_err(|error| AppError::Validation(error.to_string()))?;
    let reason = sanitize(&body.reason)?;
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM chat_messages WHERE id=$1)").bind(body.message_id).fetch_one(&state.db).await?;
    if !exists { return Err(AppError::NotFound); }
    sqlx::query("INSERT INTO chat_reports (id,reporter_user_id,message_id,reason) VALUES ($1,$2,$3,$4) ON CONFLICT (reporter_user_id,message_id) DO NOTHING").bind(Uuid::new_v4()).bind(user.user_id).bind(body.message_id).bind(reason).execute(&state.db).await?;
    audit(&state, user.user_id, "CHAT_MESSAGE_REPORTED", serde_json::json!({"message_id":body.message_id})).await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn insert_message(state: &AppState, sender: Uuid, recipient: Option<Uuid>, channel: &str, content: String) -> AppResult<ChatMessage> {
    Ok(sqlx::query_as("WITH inserted AS (INSERT INTO chat_messages (id,sender_user_id,recipient_user_id,channel,content) VALUES ($1,$2,$3,$4,$5) RETURNING id,sender_user_id,recipient_user_id,channel,content,created_at) SELECT i.id,i.sender_user_id,u.display_name AS sender_name,i.recipient_user_id,i.channel,i.content,i.created_at FROM inserted i JOIN users u ON u.id=i.sender_user_id")
        .bind(Uuid::new_v4()).bind(sender).bind(recipient).bind(channel).bind(content).fetch_one(&state.db).await?)
}

async fn enforce_rate_limit(state: &AppState, user_id: Uuid) -> AppResult<()> {
    let mut connection = state.redis.get_multiplexed_async_connection().await?;
    let result: Option<String> = redis::cmd("SET").arg(format!("chat:rate:{user_id}")).arg("1").arg("EX").arg(RATE_LIMIT_SECONDS).arg("NX").query_async(&mut connection).await?;
    if result.is_some() { Ok(()) } else { Err(AppError::Validation("aguarde antes de enviar outra mensagem".into())) }
}

async fn cache_global(state: &AppState, message: &ChatMessage) {
    let Ok(payload) = serde_json::to_string(message) else { return; };
    let Ok(mut connection) = state.redis.get_multiplexed_async_connection().await else { return; };
    let result: redis::RedisResult<()> = redis::pipe().cmd("LPUSH").arg(GLOBAL_HISTORY_KEY).arg(payload).ignore().cmd("LTRIM").arg(GLOBAL_HISTORY_KEY).arg(0).arg(GLOBAL_HISTORY_LIMIT - 1).ignore().query_async(&mut connection).await;
    if let Err(error) = result { log::warn!("não foi possível atualizar cache de chat: {error}"); }
}

async fn read_global_cache(state: &AppState, limit: i64) -> AppResult<Vec<ChatMessage>> {
    let mut connection = state.redis.get_multiplexed_async_connection().await?;
    let entries: Vec<String> = redis::cmd("LRANGE").arg(GLOBAL_HISTORY_KEY).arg(0).arg(limit - 1).query_async(&mut connection).await?;
    Ok(entries.into_iter().filter_map(|entry| serde_json::from_str(&entry).ok()).collect())
}

fn sanitize(value: &str) -> AppResult<String> {
    let text = value.trim();
    if text.is_empty() || text.chars().count() > 280 || text.chars().any(char::is_control) { return Err(AppError::Validation("mensagem inválida".into())); }
    Ok(text.to_owned())
}

async fn audit(state: &AppState, actor: Uuid, action: &str, metadata: serde_json::Value) -> AppResult<()> {
    sqlx::query("INSERT INTO audit_logs (actor_user_id,action,metadata) VALUES ($1,$2,$3)").bind(actor).bind(action).bind(metadata).execute(&state.db).await?;
    Ok(())
}

#[cfg(test)]
mod tests { use super::*; #[test] fn sanitizacao_rejeita_controle_e_espacos() { assert!(sanitize("  ").is_err()); assert!(sanitize("oi\n").is_err()); assert_eq!(sanitize(" oi ").unwrap(), "oi"); } }
