use std::sync::Arc;

use actix_web::{web, HttpResponse};
use redis::AsyncCommands;
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

use crate::{auth::middleware::AuthenticatedUser, error::AppResult, state::AppState};

const POWER_KEY: &str = "ranking:power:v1";
const MAX_PAGE_SIZE: usize = 50;

#[derive(serde::Deserialize)]
struct RankingQuery {
    #[serde(default)]
    offset: usize,
    #[serde(default = "default_limit")]
    limit: usize,
}
fn default_limit() -> usize { 20 }

#[derive(FromRow)]
struct RankingRow { user_id: Uuid, display_name: String, character_name: String, level: i16, power_rating: i64 }
#[derive(Serialize)]
struct RankingEntry { rank: usize, user_id: Uuid, display_name: String, character_name: String, level: i16, power_rating: i64 }
#[derive(Serialize)]
struct RankingPage { entries: Vec<RankingEntry>, offset: usize, limit: usize, rebuilt: bool }

pub fn configure(cfg: &mut web::ServiceConfig) { cfg.route("/rankings/power", web::get().to(power)); }

/// Atualiza a projeção após uma mudança de stats. Falhas não revertem a fonte de verdade;
/// o cache continuará reconstruível pelo PostgreSQL.
pub async fn refresh_user(state: &AppState, user_id: Uuid) {
    let score: Result<Option<i64>, sqlx::Error> = sqlx::query_scalar("SELECT cs.power_rating FROM characters c JOIN character_stats cs ON cs.character_id=c.id WHERE c.user_id=$1 AND c.is_leader=true")
        .bind(user_id).fetch_optional(&state.db).await;
    let Ok(Some(score)) = score else { return; };
    match state.redis.get_multiplexed_async_connection().await {
        Ok(mut connection) => {
            let result: redis::RedisResult<()> = connection.zadd(POWER_KEY, user_id.to_string(), score).await;
            if let Err(error) = result { log::warn!("não foi possível atualizar ranking Redis: {error}"); }
        }
        Err(error) => log::warn!("não foi possível conectar ao Redis para ranking: {error}"),
    }
}

/// Redis é apenas a projeção ordenada. Em cache vazio ela é reconstruída a partir do PostgreSQL.
async fn power(state: web::Data<Arc<AppState>>, _user: AuthenticatedUser, query: web::Query<RankingQuery>) -> AppResult<HttpResponse> {
    let limit = query.limit.clamp(1, MAX_PAGE_SIZE);
    let start = query.offset.min(isize::MAX as usize);
    let end = start.saturating_add(limit).saturating_sub(1);
    let mut connection = state.redis.get_multiplexed_async_connection().await?;
    let mut ids: Vec<String> = connection.zrevrange(POWER_KEY, start as isize, end as isize).await?;
    let mut rebuilt = false;
    if ids.is_empty() && query.offset == 0 {
        rebuild(&state, &mut connection).await?;
        ids = connection.zrevrange(POWER_KEY, start as isize, end as isize).await?;
        rebuilt = true;
    }
    let parsed_ids: Vec<Uuid> = ids.iter().filter_map(|id| Uuid::parse_str(id).ok()).collect();
    let rows = ranking_rows(&state, &parsed_ids).await?;
    // Reconstrói ranks contíguos: entradas obsoletas do cache (usuários removidos/inativos)
    // são descartadas sem abrir buracos na numeração.
    let mut entries = Vec::with_capacity(ids.len());
    for id in &ids {
        let Ok(user_id) = Uuid::parse_str(id) else { continue; };
        let Some(row) = rows.iter().find(|row| row.user_id == user_id) else { continue; };
        entries.push(RankingEntry { rank: start + entries.len() + 1, user_id, display_name: row.display_name.clone(), character_name: row.character_name.clone(), level: row.level, power_rating: row.power_rating });
    }
    Ok(HttpResponse::Ok().json(RankingPage { entries, offset: start, limit, rebuilt }))
}

async fn rebuild(state: &AppState, connection: &mut redis::aio::MultiplexedConnection) -> AppResult<()> {
    let rows: Vec<RankingRow> = sqlx::query_as("SELECT u.id AS user_id,u.display_name,c.name AS character_name,c.level,cs.power_rating FROM users u JOIN characters c ON c.user_id=u.id AND c.is_leader=true JOIN character_stats cs ON cs.character_id=c.id WHERE u.status='active' ORDER BY cs.power_rating DESC,u.id")
        .fetch_all(&state.db).await?;
    let _: () = connection.del(POWER_KEY).await?;
    if !rows.is_empty() {
        let values: Vec<(String, i64)> = rows.into_iter().map(|row| (row.user_id.to_string(), row.power_rating)).collect();
        let _: () = connection.zadd_multiple(POWER_KEY, &values).await?;
    }
    Ok(())
}

async fn ranking_rows(state: &AppState, ids: &[Uuid]) -> AppResult<Vec<RankingRow>> {
    if ids.is_empty() { return Ok(Vec::new()); }
    Ok(sqlx::query_as("SELECT u.id AS user_id,u.display_name,c.name AS character_name,c.level,cs.power_rating FROM users u JOIN characters c ON c.user_id=u.id AND c.is_leader=true JOIN character_stats cs ON cs.character_id=c.id WHERE u.id = ANY($1) AND u.status='active'")
        .bind(ids).fetch_all(&state.db).await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pagina_e_limitada() { assert_eq!(100_usize.clamp(1, MAX_PAGE_SIZE), 50); assert_eq!(0_usize.clamp(1, MAX_PAGE_SIZE), 1); }
}
