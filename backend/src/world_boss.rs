use std::sync::Arc;
use actix_web::{web, HttpResponse};
use redis::AsyncCommands;
use serde::Deserialize;
use uuid::Uuid;

use crate::{auth::middleware::AuthenticatedUser, error::{AppError, AppResult}, state::AppState};

#[derive(Deserialize)]
struct AttackRequest { damage: i64 }

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/world-boss")
        .route("/status", web::get().to(status))
        .route("/attack", web::post().to(attack))
        .route("/ranking", web::get().to(ranking))
    );
}

async fn status(state: web::Data<Arc<AppState>>, _user: AuthenticatedUser) -> AppResult<HttpResponse> {
    let row: Option<(String,i64,i64,chrono::DateTime<chrono::Utc>,chrono::DateTime<chrono::Utc>)> = sqlx::query_as("SELECT boss_name, hp, max_hp, spawns_at, expires_at FROM world_boss_state ORDER BY spawns_at DESC LIMIT 1")
        .fetch_optional(&state.db).await?;
    if let Some((name,hp,max,spawns,expires)) = row {
        Ok(HttpResponse::Ok().json(serde_json::json!({"boss_name":name,"hp":hp,"max_hp":max,"spawns_at":spawns,"expires_at":expires})))
    } else {
        Ok(HttpResponse::Ok().json(serde_json::json!({"boss_name":"Colosso Primordial","hp":15000000,"max_hp":15000000})))
    }
}

async fn attack(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, body: web::Json<AttackRequest>) -> AppResult<HttpResponse> {
    if body.damage <= 0 || body.damage > 1_000_000 { return Err(AppError::Validation("damage deve ser 1-1000000".into())); }
    let mut tx = state.db.begin().await?;
    // Calcula dano autorizado pelo servidor: base no Power Rating, mas cliente envia intenção, servidor limita
    let power: i64 = sqlx::query_scalar("SELECT cs.power_rating FROM characters c JOIN character_stats cs ON cs.character_id=c.id WHERE c.user_id=$1 AND c.is_leader=true")
        .bind(user.user_id).fetch_one(&mut *tx).await?;
    let max_damage = (power as f64 * 0.5 + 1000.0) as i64;
    let damage = body.damage.min(max_damage).max(1);
    // Atualiza HP global (shared)
    let boss_id: Uuid = sqlx::query_scalar("SELECT id FROM world_boss_state ORDER BY spawns_at DESC LIMIT 1").fetch_one(&mut *tx).await?;
    sqlx::query("UPDATE world_boss_state SET hp=GREATEST(0, hp-$2) WHERE id=$1").bind(boss_id).bind(damage).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO raid_contributions (user_id, damage) VALUES ($1,$2)").bind(user.user_id).bind(damage).execute(&mut *tx).await?;
    // Atualiza ranking Redis world boss damage ZSET
    let hp: i64 = sqlx::query_scalar("SELECT hp FROM world_boss_state WHERE id=$1").bind(boss_id).fetch_one(&mut *tx).await?;
    tx.commit().await?;
    if let Ok(mut conn) = state.redis.get_multiplexed_async_connection().await {
        let _: redis::RedisResult<()> = redis::cmd("ZINCRBY").arg("worldboss:damage:v1").arg(damage).arg(user.user_id.to_string()).query_async(&mut conn).await;
    }
    Ok(HttpResponse::Ok().json(serde_json::json!({"damage":damage,"boss_hp":hp,"max_allowed":max_damage})))
}

async fn ranking(state: web::Data<Arc<AppState>>, _user: AuthenticatedUser) -> AppResult<HttpResponse> {
    // Fonte autoritativa: PostgreSQL (Redis é cache). Soma de dano por jogador.
    let rows: Vec<(Uuid,i64)> = sqlx::query_as("SELECT user_id, SUM(damage)::bigint FROM raid_contributions GROUP BY user_id ORDER BY SUM(damage) DESC LIMIT 100")
        .fetch_all(&state.db).await?;
    let mut entries = Vec::new();
    for (i,(uid, dmg)) in rows.into_iter().enumerate() {
        if let Ok(name) = sqlx::query_scalar::<_,String>("SELECT display_name FROM users WHERE id=$1").bind(uid).fetch_one(&state.db).await {
            entries.push(serde_json::json!({"rank":i+1,"user_id":uid,"display_name":name,"damage":dmg}));
        }
    }
    // Se vazio, tenta Redis ZSET como fallback (sem WITHSCORES complexo)
    if entries.is_empty() {
        if let Ok(mut conn) = state.redis.get_multiplexed_async_connection().await {
            let ids: Vec<String> = redis::cmd("ZREVRANGE").arg("worldboss:damage:v1").arg(0).arg(99).query_async(&mut conn).await.unwrap_or_default();
            for (i, id) in ids.iter().enumerate() {
                if let Ok(uid) = Uuid::parse_str(id) {
                    if let Ok(Some(name)) = sqlx::query_scalar::<_,String>("SELECT display_name FROM users WHERE id=$1").bind(uid).fetch_optional(&state.db).await {
                        entries.push(serde_json::json!({"rank":i+1,"user_id":uid,"display_name":name,"damage":0}));
                    }
                }
            }
        }
    }
    Ok(HttpResponse::Ok().json(serde_json::json!({"entries":entries})))
}
