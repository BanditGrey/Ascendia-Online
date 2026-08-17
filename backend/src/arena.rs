use std::sync::Arc;
use actix_web::{web, HttpResponse};
use rand::Rng;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{auth::middleware::AuthenticatedUser, combat::engine::FighterStats, error::{AppError, AppResult}, state::AppState};

#[derive(FromRow, Serialize)]
struct ArenaStatus { tier: String, rating: i32, wins: i32, losses: i32, daily_fights: i32 }

fn tier_for_rating(rating: i32) -> &'static str {
    match rating {
        0..=1099 => "bronze", 1100..=1299 => "prata", 1300..=1499 => "ouro", 1500..=1699 => "platina",
        1700..=1899 => "diamante", 1900..=2099 => "mestre", 2100..=2399 => "lenda", 2400..=2699 => "divino", _ => "primordial",
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/arena")
        .route("/status", web::get().to(status))
        .route("/fight", web::post().to(fight))
        .route("/ranking", web::get().to(ranking))
    );
}

async fn status(state: web::Data<Arc<AppState>>, user: AuthenticatedUser) -> AppResult<HttpResponse> {
    let row: Option<(String,i32,i32,i32,i32)> = sqlx::query_as("SELECT tier::text, rating, wins, losses, daily_fights FROM arena_status WHERE user_id=$1")
        .bind(user.user_id).fetch_optional(&state.db).await?;
    if let Some((tier,rating,wins,losses,daily)) = row {
        Ok(HttpResponse::Ok().json(serde_json::json!({"tier":tier,"rating":rating,"wins":wins,"losses":losses,"daily_fights":daily,"remaining": (5 + vip_bonus(&state,user.user_id).await - daily).max(0)})))
    } else {
        // Seed
        sqlx::query("INSERT INTO arena_status (user_id) VALUES ($1) ON CONFLICT DO NOTHING").bind(user.user_id).execute(&state.db).await?;
        Ok(HttpResponse::Ok().json(serde_json::json!({"tier":"bronze","rating":1000,"wins":0,"losses":0,"daily_fights":0,"remaining":5})))
    }
}

async fn vip_bonus(state: &AppState, user_id: Uuid) -> i32 {
    let vip: i16 = sqlx::query_scalar("SELECT vip_level FROM users WHERE id=$1").bind(user_id).fetch_one(&state.db).await.unwrap_or(0);
    match vip {
        0..=2 => 0, 3..=5 => 1, 6..=8 => 2, 9..=10 => 5, 11..=13 => 10, 14..=15 => 15, _=>0
    }
}

async fn fight(state: web::Data<Arc<AppState>>, user: AuthenticatedUser) -> AppResult<HttpResponse> {
    let mut tx = state.db.begin().await?;
    // Reset diário
    sqlx::query("UPDATE arena_status SET daily_fights=0, last_reset=CURRENT_DATE WHERE user_id=$1 AND last_reset < CURRENT_DATE").bind(user.user_id).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO arena_status (user_id) VALUES ($1) ON CONFLICT (user_id) DO NOTHING").bind(user.user_id).execute(&mut *tx).await?;
    let (rating, daily, tier): (i32,i32,String) = sqlx::query_as("SELECT rating, daily_fights, tier::text FROM arena_status WHERE user_id=$1 FOR UPDATE").bind(user.user_id).fetch_one(&mut *tx).await?;
    let bonus = vip_bonus(&state, user.user_id).await;
    let max = 5 + bonus;
    if daily >= max { return Err(AppError::Validation(format!("limite diário {max} atingido (VIP +{bonus})"))); }
    // Power do atacante
    let my_power: i64 = sqlx::query_scalar("SELECT cs.power_rating FROM characters c JOIN character_stats cs ON cs.character_id=c.id WHERE c.user_id=$1 AND c.is_leader=true").bind(user.user_id).fetch_one(&mut *tx).await?;
    // Matchmaking por Power Rating ±25%
    let low = (my_power as f64 * 0.75) as i64;
    let high = (my_power as f64 * 1.25) as i64;
    let opponent: Option<(Uuid,i64,i32)> = sqlx::query_as("SELECT u.id, cs.power_rating, a.rating FROM users u JOIN characters c ON c.user_id=u.id AND c.is_leader=true JOIN character_stats cs ON cs.character_id=c.id JOIN arena_status a ON a.user_id=u.id WHERE u.id!=$1 AND cs.power_rating BETWEEN $2 AND $3 ORDER BY RANDOM() LIMIT 1")
        .bind(user.user_id).bind(low).bind(high).fetch_optional(&mut *tx).await?;
    // Se não há oponente, cria bot
    let (opp_id, opp_power, opp_rating) = opponent.unwrap_or((Uuid::nil(), (my_power as f64 * 0.9) as i64, rating));
    // Simulação simples: power + rng
    let mut rng = rand::thread_rng();
    let my_roll = my_power as f64 * rng.gen_range(0.85..1.15);
    let opp_roll = opp_power as f64 * rng.gen_range(0.85..1.15);
    let victory = my_roll > opp_roll;
    let winner = if victory { user.user_id } else { opp_id };
    let delta = 15;
    let new_rating = if victory { rating + delta } else { (rating - delta).max(0) };
    let new_tier = tier_for_rating(new_rating);
    sqlx::query("UPDATE arena_status SET rating=$2, tier=$3::text::arena_tier, wins=wins+$4, losses=losses+$5, daily_fights=daily_fights+1, updated_at=now() WHERE user_id=$1")
        .bind(user.user_id).bind(new_rating).bind(new_tier).bind(if victory {1} else {0}).bind(if victory {0} else {1}).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO arena_matches (attacker_user_id, defender_user_id, winner_user_id, attacker_power, defender_power) VALUES ($1,$2,$3,$4,$5)")
        .bind(user.user_id).bind(opp_id).bind(winner).bind(my_power).bind(opp_power).execute(&mut *tx).await?;
    // Atualiza ranking Redis
    tx.commit().await?;
    if let Ok(mut conn) = state.redis.get_multiplexed_async_connection().await {
        let _: redis::RedisResult<()> = redis::cmd("ZADD").arg("ranking:arena:v1").arg(new_rating).arg(user.user_id.to_string()).query_async(&mut conn).await;
    }
    Ok(HttpResponse::Ok().json(serde_json::json!({"victory":victory,"my_power":my_power,"opp_power":opp_power,"new_rating":new_rating,"tier":new_tier,"opp_id":opp_id})))
}

async fn ranking(state: web::Data<Arc<AppState>>, _user: AuthenticatedUser) -> AppResult<HttpResponse> {
    let mut conn = state.redis.get_multiplexed_async_connection().await?;
    let ids: Vec<String> = redis::cmd("ZREVRANGE").arg("ranking:arena:v1").arg(0).arg(19).query_async(&mut conn).await?;
    let mut entries = Vec::new();
    for (i, id) in ids.iter().enumerate() {
        if let Ok(uid) = Uuid::parse_str(id) {
            if let Ok(Some((name, rating, tier))) = sqlx::query_as::<_,(String,i32,String)>("SELECT u.display_name, a.rating, a.tier::text FROM users u JOIN arena_status a ON a.user_id=u.id WHERE u.id=$1").bind(uid).fetch_optional(&state.db).await {
                entries.push(serde_json::json!({"rank":i+1,"user_id":uid,"display_name":name,"rating":rating,"tier":tier}));
            }
        }
    }
    Ok(HttpResponse::Ok().json(serde_json::json!({"entries":entries})))
}
