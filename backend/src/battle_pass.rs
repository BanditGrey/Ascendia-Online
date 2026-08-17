use std::sync::Arc;

use actix_web::{web, HttpResponse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{auth::middleware::AuthenticatedUser, error::{AppError, AppResult}, state::AppState};

#[derive(FromRow, Serialize, Clone)]
struct Season { id: Uuid, name: String, starts_at: DateTime<Utc>, ends_at: DateTime<Utc>, premium_cost: i32, is_active: bool }

#[derive(FromRow, Serialize)]
struct Progress { season_id: Uuid, level: i16, xp: i64, premium: bool }

#[derive(Serialize)]
struct BattlePassView {
    season: Season,
    progress: Progress,
    rewards: Vec<LevelReward>,
    next_level_xp: i64,
}

#[derive(Serialize, Clone)]
struct LevelReward {
    level: i16,
    free: String,
    premium: String,
}

fn rewards_for(level: i16) -> LevelReward {
    match level {
        1  => LevelReward { level, free: "500 Gold".into(), premium: "2000 Gold".into() },
        5  => LevelReward { level, free: "Frag Asa ×10".into(), premium: "Frag Asa ×30".into() },
        10 => LevelReward { level, free: "Box Raro".into(), premium: "Box Épico".into() },
        15 => LevelReward { level, free: "Frag Mont ×10".into(), premium: "Frag Mont ×30".into() },
        20 => LevelReward { level, free: "100 Diamantes".into(), premium: "500 Diamantes".into() },
        25 => LevelReward { level, free: "Frag Pet ×10".into(), premium: "Frag Pet ×30".into() },
        30 => LevelReward { level, free: "Runa Box".into(), premium: "Runa Box Épica".into() },
        35 => LevelReward { level, free: "Essência ×1".into(), premium: "Essência ×3".into() },
        40 => LevelReward { level, free: "Box Épico".into(), premium: "Box Lendário".into() },
        45 => LevelReward { level, free: "200 Diamantes".into(), premium: "1000 Diamantes".into() },
        50 => LevelReward { level, free: "Título+Frame".into(), premium: "SKIN EXCLUSIVA + Aura+Trail + 2000 Diamantes (NUNCA retorna)".into() },
        _  => LevelReward { level, free: "Gold/fragmentos".into(), premium: "Gold/fragmentos premium".into() },
    }
}

#[derive(Deserialize)]
struct ClaimRequest { level: i16 }

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/battle-pass")
        .route("", web::get().to(get_pass))
        .route("/premium", web::post().to(activate_premium))
        .route("/claim", web::post().to(claim))
    );
}

async fn get_pass(state: web::Data<Arc<AppState>>, user: AuthenticatedUser) -> AppResult<HttpResponse> {
    let season: Season = sqlx::query_as("SELECT id,name,starts_at,ends_at,premium_cost,is_active FROM battle_pass_seasons WHERE is_active=true ORDER BY starts_at DESC LIMIT 1")
        .fetch_one(&state.db).await.map_err(|_| AppError::NotFound)?;
    let progress: Option<Progress> = sqlx::query_as("SELECT season_id,level,xp,premium FROM battle_pass_progress WHERE user_id=$1 AND season_id=$2")
        .bind(user.user_id).bind(season.id).fetch_optional(&state.db).await?;
    let progress = progress.unwrap_or(Progress { season_id: season.id, level: 0, xp: 0, premium: false });
    let rewards: Vec<LevelReward> = (1..=50).map(rewards_for).collect();
    let next_level_xp = ((progress.level as i64 + 1) * 1000) as i64;
    Ok(HttpResponse::Ok().json(BattlePassView { season, progress, rewards, next_level_xp }))
}

async fn activate_premium(state: web::Data<Arc<AppState>>, user: AuthenticatedUser) -> AppResult<HttpResponse> {
    let season: Season = sqlx::query_as("SELECT id,name,starts_at,ends_at,premium_cost,is_active FROM battle_pass_seasons WHERE is_active=true LIMIT 1")
        .fetch_one(&state.db).await.map_err(|_| AppError::NotFound)?;
    let mut tx = state.db.begin().await?;
    // Custo em Diamantes (autoritative)
    let diamonds: i64 = sqlx::query_scalar("SELECT diamonds FROM users WHERE id=$1 FOR UPDATE").bind(user.user_id).fetch_one(&mut *tx).await?;
    if diamonds < season.premium_cost as i64 { return Err(AppError::Validation(format!("diamantes insuficientes: precisa {}", season.premium_cost))); }
    sqlx::query("UPDATE users SET diamonds=diamonds-$2 WHERE id=$1").bind(user.user_id).bind(season.premium_cost as i64).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO battle_pass_progress (user_id,season_id,premium) VALUES ($1,$2,true) ON CONFLICT (user_id,season_id) DO UPDATE SET premium=true, updated_at=now()")
        .bind(user.user_id).bind(season.id).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO audit_logs (actor_user_id,action,metadata) VALUES ($1,'BATTLEPASS_PREMIUM',$2)").bind(user.user_id).bind(serde_json::json!({"season_id":season.id})).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"premium":true,"season_id":season.id})))
}

async fn claim(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, body: web::Json<ClaimRequest>) -> AppResult<HttpResponse> {
    if !(1..=50).contains(&body.level) { return Err(AppError::Validation("level deve ser 1-50".into())); }
    let season: Season = sqlx::query_as("SELECT id,name,starts_at,ends_at,premium_cost,is_active FROM battle_pass_seasons WHERE is_active=true LIMIT 1")
        .fetch_one(&state.db).await.map_err(|_| AppError::NotFound)?;
    let prog: Option<Progress> = sqlx::query_as("SELECT season_id,level,xp,premium FROM battle_pass_progress WHERE user_id=$1 AND season_id=$2")
        .bind(user.user_id).bind(season.id).fetch_optional(&state.db).await?;
    let prog = prog.unwrap_or(Progress { season_id: season.id, level: 0, xp: 0, premium: false });
    let required_xp = body.level as i64 * 1000;
    if prog.xp < required_xp { return Err(AppError::Validation(format!("XP insuficiente: {}/{}", prog.xp, required_xp))); }
    if prog.level < body.level { return Err(AppError::Validation("level ainda não alcançado".into())); }
    // Auditar claim (recompensas são concedidas via fragmentos/itens; aqui apenas auditoria + idempotência simples)
    sqlx::query("INSERT INTO audit_logs (actor_user_id,action,metadata) VALUES ($1,'BATTLEPASS_CLAIM',$2)")
        .bind(user.user_id).bind(serde_json::json!({"season_id":season.id,"level":body.level,"premium":prog.premium})).execute(&state.db).await?;
    let reward = rewards_for(body.level);
    Ok(HttpResponse::Ok().json(serde_json::json!({"claimed":body.level,"reward":reward,"premium":prog.premium})))
}
