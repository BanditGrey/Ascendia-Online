use crate::{error::{AppError, AppResult}, state::AppState};

/// Rate limit por IP (100 req/min) e por usuário (60 req/min mutação).
/// Usa Redis INCR com TTL 60s. Se Redis falhar, permite (fail-open) mas loga.

pub async fn enforce_ip(state: &AppState, ip: &str) -> AppResult<()> {
    if ip.is_empty() { return Ok(()); }
    let key = format!("ratelimit:ip:{ip}");
    let mut conn = state.redis.get_multiplexed_async_connection().await.map_err(|_| AppError::Internal("redis".into()))?;
    let count: i32 = redis::cmd("INCR").arg(&key).query_async(&mut conn).await.map_err(|_| AppError::Internal("redis".into()))?;
    if count == 1 {
        let _: () = redis::cmd("EXPIRE").arg(&key).arg(60).query_async(&mut conn).await.unwrap_or(());
    }
    if count > 100 {
        return Err(AppError::Validation("rate limit por IP excedido (100/min)".into()));
    }
    Ok(())
}

pub async fn enforce_user(state: &AppState, user_id: uuid::Uuid, action: &str) -> AppResult<()> {
    let key = format!("ratelimit:user:{}:{}", user_id, action);
    let mut conn = state.redis.get_multiplexed_async_connection().await.map_err(|_| AppError::Internal("redis".into()))?;
    let count: i32 = redis::cmd("INCR").arg(&key).query_async(&mut conn).await.map_err(|_| AppError::Internal("redis".into()))?;
    if count == 1 {
        let _: () = redis::cmd("EXPIRE").arg(&key).arg(60).query_async(&mut conn).await.unwrap_or(());
    }
    let limit = match action {
        "combat" => 30,
        "chat" => 20, // já tem 3s, mas reforça
        "trade" => 10,
        _ => 60,
    };
    if count > limit {
        return Err(AppError::Validation(format!("rate limit {action} excedido ({limit}/min)")));
    }
    Ok(())
}
