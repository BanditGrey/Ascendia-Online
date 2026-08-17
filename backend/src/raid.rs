use std::sync::Arc;
use actix_web::{web, HttpResponse};
use rand::Rng;
use redis::AsyncCommands;
use uuid::Uuid;

use crate::{auth::middleware::AuthenticatedUser, error::{AppError, AppResult}, state::AppState};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/raid")
        .route("/status", web::get().to(status))
        .route("/attack", web::post().to(attack))
        .route("/ranking", web::get().to(ranking))
    );
}

async fn status(state: web::Data<Arc<AppState>>, _user: AuthenticatedUser) -> AppResult<HttpResponse> {
    let row: Option<(Uuid,String,i64,i64,String)> = sqlx::query_as("SELECT id, name, hp, max_hp, status::text FROM raid_bosses ORDER BY created_at DESC LIMIT 1").fetch_optional(&state.db).await?;
    if let Some((id,name,hp,max,status)) = row {
        let total: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(damage),0)::bigint FROM raid_hits WHERE raid_id=$1").bind(id).fetch_one(&state.db).await?;
        Ok(HttpResponse::Ok().json(serde_json::json!({"id":id,"name":name,"hp":hp,"max_hp":max,"status":status,"total_damage":total,"resets":"Segunda e Quinta"})))
    } else {
        Ok(HttpResponse::Ok().json(serde_json::json!({"name":"Dragão Ancião","hp":50000000})))
    }
}

async fn attack(state: web::Data<Arc<AppState>>, user: AuthenticatedUser) -> AppResult<HttpResponse> {
    let mut tx = state.db.begin().await?;
    let raid_id: Uuid = sqlx::query_scalar("SELECT id FROM raid_bosses WHERE status='active' ORDER BY created_at DESC LIMIT 1").fetch_optional(&mut *tx).await?.ok_or(AppError::NotFound)?;
    let hp: i64 = sqlx::query_scalar("SELECT hp FROM raid_bosses WHERE id=$1 FOR UPDATE").bind(raid_id).fetch_one(&mut *tx).await?;
    if hp <= 0 { return Err(AppError::Validation("raid já derrotado, aguarde reset Seg/Qui".into())); }
    let guild_id: Option<Uuid> = sqlx::query_scalar("SELECT guild_id FROM guild_members WHERE user_id=$1").bind(user.user_id).fetch_optional(&mut *tx).await?;
    let power: i64 = sqlx::query_scalar("SELECT cs.power_rating FROM characters c JOIN character_stats cs ON cs.character_id=c.id WHERE c.user_id=$1 AND c.is_leader=true").bind(user.user_id).fetch_one(&mut *tx).await?;
    let mut rng = rand::thread_rng();
    let damage = (power as f64 * rng.gen_range(0.4..0.7) + 500.0) as i64;
    sqlx::query("UPDATE raid_bosses SET hp=GREATEST(0, hp-$2) WHERE id=$1").bind(raid_id).bind(damage).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO raid_hits (raid_id, user_id, guild_id, damage) VALUES ($1,$2,$3,$4)").bind(raid_id).bind(user.user_id).bind(guild_id).bind(damage).execute(&mut *tx).await?;
    let remaining: i64 = sqlx::query_scalar("SELECT hp FROM raid_bosses WHERE id=$1").bind(raid_id).fetch_one(&mut *tx).await?;
    let my_total: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(damage),0)::bigint FROM raid_hits WHERE raid_id=$1 AND user_id=$2").bind(raid_id).bind(user.user_id).fetch_one(&mut *tx).await?;
    if remaining == 0 {
        sqlx::query("UPDATE raid_bosses SET status='defeated' WHERE id=$1").bind(raid_id).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    if let Ok(mut conn) = state.redis.get_multiplexed_async_connection().await {
        let _: redis::RedisResult<()> = redis::cmd("ZINCRBY").arg(format!("raid:{raid_id}:damage")).arg(damage).arg(user.user_id.to_string()).query_async(&mut conn).await;
    }
    Ok(HttpResponse::Ok().json(serde_json::json!({"damage":damage,"remaining":remaining,"my_total":my_total,"defeated":remaining==0})))
}

async fn ranking(state: web::Data<Arc<AppState>>, _user: AuthenticatedUser) -> AppResult<HttpResponse> {
    let raid_id: Option<Uuid> = sqlx::query_scalar("SELECT id FROM raid_bosses ORDER BY created_at DESC LIMIT 1").fetch_optional(&state.db).await?;
    let raid_id = raid_id.ok_or(AppError::NotFound)?;
    let rows: Vec<(Uuid,i64)> = sqlx::query_as("SELECT user_id, SUM(damage)::bigint as dmg FROM raid_hits WHERE raid_id=$1 GROUP BY user_id ORDER BY dmg DESC LIMIT 20").bind(raid_id).fetch_all(&state.db).await?;
    let mut entries = Vec::new();
    for (uid,dmg) in rows {
        if let Ok(name) = sqlx::query_scalar::<_,String>("SELECT display_name FROM users WHERE id=$1").bind(uid).fetch_one(&state.db).await {
            entries.push(serde_json::json!({"user_id":uid,"display_name":name,"damage":dmg}));
        }
    }
    Ok(HttpResponse::Ok().json(serde_json::json!({"raid_id":raid_id,"entries":entries})))
}
