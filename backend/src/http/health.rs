use std::sync::Arc;
use actix_web::{web, HttpResponse};
use serde::Serialize;
use crate::{error::AppResult, state::AppState};

#[derive(Serialize)]
struct Health { status: &'static str, database: &'static str, redis: &'static str }

pub async fn health(state: web::Data<Arc<AppState>>) -> AppResult<HttpResponse> {
    sqlx::query_scalar::<_, i32>("SELECT 1").fetch_one(&state.db).await?;
    let mut connection = state.redis.get_multiplexed_async_connection().await?;
    let _: String = redis::cmd("PING").query_async(&mut connection).await?;
    Ok(HttpResponse::Ok().json(Health { status: "ok", database: "ok", redis: "ok" }))
}

pub async fn metrics(state: web::Data<Arc<AppState>>) -> AppResult<HttpResponse> {
    let counters: Vec<(String,i64)> = sqlx::query_as("SELECT name, value FROM metrics_counters ORDER BY name").fetch_all(&state.db).await.unwrap_or_default();
    let mut out = String::from("# HELP ascendia_requests_total Total requests\n# TYPE ascendia_requests_total counter\n");
    for (name, val) in counters {
        out.push_str(&format!("ascendia_{} {}\n", name, val));
    }
    // Adiciona métricas de sistema mock
    out.push_str("# HELP ascendia_online 1 if up\n# TYPE ascendia_online gauge\nascendia_online 1\n");
    // Health check counts
    let users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users").fetch_one(&state.db).await.unwrap_or(0);
    out.push_str(&format!("ascendia_users_total {}\n", users));
    let guilds: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM guilds").fetch_one(&state.db).await.unwrap_or(0);
    out.push_str(&format!("ascendia_guilds_total {}\n", guilds));
    Ok(HttpResponse::Ok().content_type("text/plain; version=0.0.4").body(out))
}
