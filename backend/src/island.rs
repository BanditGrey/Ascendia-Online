use std::sync::Arc;
use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::{auth::middleware::AuthenticatedUser, error::{AppError, AppResult}, state::AppState};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/island")
        .route("/status", web::get().to(status))
        .route("/unlock", web::post().to(unlock))
        .route("/enter/{stage}", web::post().to(enter))
    );
}

async fn status(state: web::Data<Arc<AppState>>, user: AuthenticatedUser) -> AppResult<HttpResponse> {
    let max_stage: i16 = sqlx::query_scalar("SELECT COALESCE(max_stage,0)::smallint FROM stage_progress WHERE user_id=$1").bind(user.user_id).fetch_one(&state.db).await?;
    let gold: i64 = sqlx::query_scalar("SELECT gold FROM users WHERE id=$1").bind(user.user_id).fetch_one(&state.db).await?;
    let prog: Option<(bool,i16)> = sqlx::query_as("SELECT unlocked, max_stage FROM island_progress WHERE user_id=$1 AND island_code='abyss_island'").bind(user.user_id).fetch_optional(&state.db).await?;
    let (unlocked, island_max) = prog.unwrap_or((false,500));
    let can_unlock = max_stage >= 500 && gold >= 5000;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "island":"abyss_island","name":"Ilha do Abismo Profundo","range":"501-550","theme":"Abismo aquático bioluminescente",
        "requirement":"Fase 500 + 5000 Gold","max_stage":max_stage,"gold":gold,"can_unlock":can_unlock,"unlocked":unlocked,"island_max":island_max,
        "mobs":["abyssal_horror","deep_one","leviathan_spawn"],"boss":"Leviatã Ancião 550","loot":"Lâmina Abissal, Coração do Abismo, Coroa do Leviatã, Skins Abissais"
    })))
}

async fn unlock(state: web::Data<Arc<AppState>>, user: AuthenticatedUser) -> AppResult<HttpResponse> {
    let mut tx = state.db.begin().await?;
    let max_stage: i16 = sqlx::query_scalar("SELECT COALESCE(max_stage,0)::smallint FROM stage_progress WHERE user_id=$1 FOR UPDATE").bind(user.user_id).fetch_one(&mut *tx).await?;
    if max_stage < 500 { return Err(AppError::Validation("requer Cap.10 Fase 500 completa".into())); }
    let gold: i64 = sqlx::query_scalar("SELECT gold FROM users WHERE id=$1 FOR UPDATE").bind(user.user_id).fetch_one(&mut *tx).await?;
    if gold < 5000 { return Err(AppError::Validation("5000 Gold necessários para desbloquear Ilha".into())); }
    let already: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM island_progress WHERE user_id=$1 AND island_code='abyss_island' AND unlocked=true)").bind(user.user_id).fetch_one(&mut *tx).await?;
    if already { return Err(AppError::Validation("Ilha já desbloqueada".into())); }
    sqlx::query("UPDATE users SET gold=gold-5000 WHERE id=$1").bind(user.user_id).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO island_progress (user_id, island_code, unlocked, unlocked_at, max_stage) VALUES ($1,'abyss_island',true,now(),501) ON CONFLICT (user_id,island_code) DO UPDATE SET unlocked=true, unlocked_at=now()")
        .bind(user.user_id).execute(&mut *tx).await?;
    // Recompensa: 1 skin abissal
    let skin: Option<Uuid> = sqlx::query_scalar("SELECT id FROM cosmetic_skins WHERE skin_code='wings_t8_island_abyss'").fetch_optional(&mut *tx).await?;
    if let Some(sid)=skin { sqlx::query("INSERT INTO user_cosmetic_skins (user_id, skin_id) VALUES ($1,$2) ON CONFLICT DO NOTHING").bind(user.user_id).bind(sid).execute(&mut *tx).await?; }
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"unlocked":"abyss_island","stage":501,"reward":"wings_t8_island_abyss"})))
}

async fn enter(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, stage: web::Path<u16>) -> AppResult<HttpResponse> {
    let s = stage.into_inner();
    if !(501..=550).contains(&s) { return Err(AppError::Validation("Ilha 501-550 apenas".into())); }
    let unlocked: bool = sqlx::query_scalar("SELECT COALESCE((SELECT unlocked FROM island_progress WHERE user_id=$1 AND island_code='abyss_island'), false)").bind(user.user_id).fetch_one(&state.db).await?;
    if !unlocked { return Err(AppError::Validation("desbloqueie a Ilha primeiro (5000 Gold)".into())); }
    // Reusa combat engine com stage 501-550, mobs abissais
    let res: serde_json::Value = serde_json::json!({"stage":s,"island":"abyss_island","enemies":["abyssal_horror","deep_one"],"boss": s==550,"note":"Use POST /api/v1/combat/start com stage 501-550 — mobs abissais com scaling"});
    Ok(HttpResponse::Ok().json(res))
}
