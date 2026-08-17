use std::sync::Arc;

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{auth::middleware::AuthenticatedUser, error::{AppError, AppResult}, state::AppState};

const VIP_THRESHOLDS: [i64; 16] = [0,100,300,600,1000,1500,2500,4000,6000,8500,12000,16000,21000,27000,34000,42000];

fn level_for_points(points: i64) -> i16 {
    let mut lvl = 0;
    for (i, &thr) in VIP_THRESHOLDS.iter().enumerate() {
        if points >= thr { lvl = i as i16; } else { break; }
    }
    lvl.min(15)
}

fn benefits_for(level: i16) -> Vec<&'static str> {
    match level {
        0 => vec!["Base do jogo"],
        1 => vec!["+5% EXP","+1 Dungeon/dia","chat colorido"],
        2 => vec!["+10% EXP","+5% Drop Rate","Auto-loot"],
        3 => vec!["+15% EXP","+10% Drop","+1 Arena/dia"],
        4 => vec!["+20% EXP","+15% Drop","Skip animações"],
        5 => vec!["+25% EXP","+20% Drop","+1 Expedição","Frame Prata"],
        6 => vec!["+30% EXP","+25% Drop","+2 Arena"],
        7 => vec!["+40% EXP","+30% Drop","+2 Dungeon","VIP Chat"],
        8 => vec!["+50% EXP","+35% Drop","+1 Pet slot","Frame Ouro"],
        9 => vec!["+60% EXP","+40% Drop","+3 Arena"],
        10 => vec!["+75% EXP","+50% Drop","Título Patrono","Aura Prata"],
        11 => vec!["+90% EXP","+60% Drop","+3 Dungeon"],
        12 => vec!["+100% EXP","+75% Drop","Asas VIP Douradas"],
        13 => vec!["+120% EXP","+90% Drop","Montaria Dragão Dourado"],
        14 => vec!["+150% EXP","+100% Drop","+5 Arena","Trail VIP"],
        15 => vec!["+200% EXP","+150% Drop","Skin Imperador Dourado","Crown VIP","Aura Dourada","Offline 24h"],
        _ => vec![],
    }
}

#[derive(Serialize)]
struct VipStatus {
    vip_level: i16,
    vip_points: i64,
    next_level_points: Option<i64>,
    benefits: Vec<String>,
    all_benefits: Vec<Vec<String>>,
}

#[derive(Deserialize)]
struct GrantRequest { points: i64 }

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/vip")
        .route("/status", web::get().to(status))
        .route("/grant", web::post().to(grant))
    );
}

async fn status(state: web::Data<Arc<AppState>>, user: AuthenticatedUser) -> AppResult<HttpResponse> {
    let row: Option<(i16, i64)> = sqlx::query_as("SELECT u.vip_level, COALESCE(v.vip_points,0) FROM users u LEFT JOIN vip_progress v ON v.user_id=u.id WHERE u.id=$1")
        .bind(user.user_id).fetch_optional(&state.db).await?;
    let (level_db, points) = row.unwrap_or((0,0));
    let calc_level = level_for_points(points);
    let level = calc_level.max(level_db);
    let next = if level < 15 { Some(VIP_THRESHOLDS[(level+1) as usize]) } else { None };
    let benefits: Vec<String> = benefits_for(level).into_iter().map(|s| s.to_string()).collect();
    let all: Vec<Vec<String>> = (0..=level).map(|l| benefits_for(l).into_iter().map(|s| s.to_string()).collect()).collect();
    Ok(HttpResponse::Ok().json(VipStatus { vip_level: level, vip_points: points, next_level_points: next, benefits, all_benefits: all }))
}

async fn grant(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, body: web::Json<GrantRequest>) -> AppResult<HttpResponse> {
    if body.points <= 0 || body.points > 100_000 { return Err(AppError::Validation("points deve estar entre 1 e 100000".into())); }
    let mut tx = state.db.begin().await?;
    sqlx::query("INSERT INTO vip_progress (user_id, vip_points) VALUES ($1,$2) ON CONFLICT (user_id) DO UPDATE SET vip_points=vip_progress.vip_points+EXCLUDED.vip_points, updated_at=now()")
        .bind(user.user_id).bind(body.points).execute(&mut *tx).await?;
    let points: i64 = sqlx::query_scalar("SELECT vip_points FROM vip_progress WHERE user_id=$1").bind(user.user_id).fetch_one(&mut *tx).await?;
    let level = level_for_points(points);
    sqlx::query("UPDATE users SET vip_level=$2 WHERE id=$1").bind(user.user_id).bind(level).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO audit_logs (actor_user_id,action,metadata) VALUES ($1,'VIP_GRANTED',$2)").bind(user.user_id).bind(serde_json::json!({"points":body.points,"total":points,"level":level})).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"vip_level":level,"vip_points":points,"granted":body.points})))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn vip_levels_progridem() {
        assert_eq!(level_for_points(0),0);
        assert_eq!(level_for_points(100),1);
        assert_eq!(level_for_points(42000),15);
        assert_eq!(level_for_points(99999),15);
    }
}
