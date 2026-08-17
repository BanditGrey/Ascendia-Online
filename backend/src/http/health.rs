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
