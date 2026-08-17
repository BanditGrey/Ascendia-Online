use std::sync::Arc;
use actix_web::{web, HttpResponse};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{auth::middleware::AuthenticatedUser, error::{AppError, AppResult}, player::progression::grant_leader_experience, state::AppState};

#[derive(Deserialize)]
struct RunRequest { #[serde(rename="type")] dungeon_type: String }

#[derive(Serialize)]
struct DungeonStatus { exp_runs: i64, material_runs: i64, equipment_runs: i64, remaining: i64, max: i64 }

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/dungeons")
        .route("/status", web::get().to(status))
        .route("/run", web::post().to(run))
    );
}

async fn max_for_user(state: &AppState, user_id: Uuid) -> i64 {
    let vip: i16 = sqlx::query_scalar("SELECT vip_level FROM users WHERE id=$1").bind(user_id).fetch_one(&state.db).await.unwrap_or(0);
    if vip >= 10 { 10 } else if vip >= 5 { 6 } else { 3 }
}

async fn status(state: web::Data<Arc<AppState>>, user: AuthenticatedUser) -> AppResult<HttpResponse> {
    let max = max_for_user(&state, user.user_id).await;
    let today = Utc::now().date_naive();
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM dungeon_runs WHERE user_id=$1 AND run_date=$2").bind(user.user_id).bind(today).fetch_one(&state.db).await?;
    let exp: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM dungeon_runs WHERE user_id=$1 AND run_date=$2 AND dungeon_type='exp'").bind(user.user_id).bind(today).fetch_one(&state.db).await?;
    let material: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM dungeon_runs WHERE user_id=$1 AND run_date=$2 AND dungeon_type='material'").bind(user.user_id).bind(today).fetch_one(&state.db).await?;
    let equip: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM dungeon_runs WHERE user_id=$1 AND run_date=$2 AND dungeon_type='equipment'").bind(user.user_id).bind(today).fetch_one(&state.db).await?;
    Ok(HttpResponse::Ok().json(DungeonStatus { exp_runs:exp, material_runs:material, equipment_runs:equip, remaining: (max-count).max(0), max }))
}

async fn run(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, body: web::Json<RunRequest>) -> AppResult<HttpResponse> {
    let dtype = body.dungeon_type.to_lowercase();
    if !["exp","material","equipment"].contains(&dtype.as_str()) { return Err(AppError::Validation("tipo deve ser exp, material ou equipment".into())); }
    let mut tx = state.db.begin().await?;
    let max = max_for_user(&state, user.user_id).await;
    let today = Utc::now().date_naive();
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM dungeon_runs WHERE user_id=$1 AND run_date=$2 FOR UPDATE").bind(user.user_id).bind(today).fetch_one(&mut *tx).await?;
    if count >= max { return Err(AppError::Validation(format!("limite diário {max} atingido"))); }
    // Recompensas autoritativas baseadas em Power Rating (auto-balanceada)
    let power: i64 = sqlx::query_scalar("SELECT cs.power_rating FROM characters c JOIN character_stats cs ON cs.character_id=c.id WHERE c.user_id=$1 AND c.is_leader=true").bind(user.user_id).fetch_one(&mut *tx).await?;
    let (gold, xp, frags) = match dtype.as_str() {
        "exp" => (0, 100 + power/20, 0),
        "material" => (0, 20, 5),
        "equipment" => (50, 30, 2),
        _=> (0,0,0),
    };
    sqlx::query("INSERT INTO dungeon_runs (user_id, dungeon_type, run_date) VALUES ($1,$2::text::dungeon_type,$3)").bind(user.user_id).bind(&dtype).bind(today).execute(&mut *tx).await?;
    if gold>0 { sqlx::query("UPDATE users SET gold=gold+$2 WHERE id=$1").bind(user.user_id).bind(gold).execute(&mut *tx).await?; }
    if frags>0 {
        for kind in ["wings","mount","pet"] {
            sqlx::query("INSERT INTO cosmetic_progress (user_id,cosmetic_type,fragments) VALUES ($1,$2,$3) ON CONFLICT (user_id,cosmetic_type) DO UPDATE SET fragments=cosmetic_progress.fragments+EXCLUDED.fragments")
                .bind(user.user_id).bind(kind).bind(frags).execute(&mut *tx).await?;
        }
    }
    if xp>0 { grant_leader_experience(&mut tx, user.user_id, xp).await?; }
    // Drop equipamento se equipment
    if dtype=="equipment" {
        sqlx::query("INSERT INTO inventory_items (user_id, template_id) SELECT $1, id FROM item_templates ORDER BY RANDOM() LIMIT 1").bind(user.user_id).execute(&mut *tx).await?;
    }
    sqlx::query("INSERT INTO audit_logs (actor_user_id,action,metadata) VALUES ($1,'DUNGEON_RUN',$2)").bind(user.user_id).bind(serde_json::json!({"type":dtype,"gold":gold,"xp":xp})).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"dungeon_type":dtype,"gold":gold,"xp":xp,"frags":frags})))
}
