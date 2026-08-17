use std::sync::Arc;
use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::{auth::middleware::AuthenticatedUser, error::{AppError, AppResult}, state::AppState};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/island")
        .route("/status", web::get().to(status))
        .route("/unlock", web::post().to(unlock))
        .route("/unlock/{island_code}", web::post().to(unlock_specific))
        .route("/enter/{stage}", web::post().to(enter))
    );
}

async fn status(state: web::Data<Arc<AppState>>, user: AuthenticatedUser) -> AppResult<HttpResponse> {
    let max_stage: i16 = sqlx::query_scalar("SELECT COALESCE(max_stage,0)::smallint FROM stage_progress WHERE user_id=$1").bind(user.user_id).fetch_one(&state.db).await?;
    let gold: i64 = sqlx::query_scalar("SELECT gold FROM users WHERE id=$1").bind(user.user_id).fetch_one(&state.db).await?;
    let vip: i16 = sqlx::query_scalar("SELECT vip_level FROM users WHERE id=$1").bind(user.user_id).fetch_one(&state.db).await.unwrap_or(0);
    let prog_abyss: Option<(bool,i16)> = sqlx::query_as("SELECT unlocked, max_stage FROM island_progress WHERE user_id=$1 AND island_code='abyss_island'").bind(user.user_id).fetch_optional(&state.db).await?;
    let (unlocked_abyss, island_max_abyss) = prog_abyss.unwrap_or((false,500));
    let prog_gold: Option<(bool,i16)> = sqlx::query_as("SELECT unlocked, max_stage FROM island_progress WHERE user_id=$1 AND island_code='golden_kingdom'").bind(user.user_id).fetch_optional(&state.db).await?;
    let (unlocked_gold, island_max_gold) = prog_gold.unwrap_or((false,550));
    Ok(HttpResponse::Ok().json(serde_json::json!([
        {"island":"abyss_island","name":"Ilha do Abismo Profundo","range":"501-550","theme":"Abismo aquático bioluminescente","requirement":"Fase 500 + 5000 Gold","max_stage":max_stage,"gold":gold,"can_unlock":max_stage>=500 && gold>=5000,"unlocked":unlocked_abyss,"island_max":island_max_abyss,"mobs":["abyssal_horror","deep_one","leviathan_spawn"],"boss":"Leviatã Ancião 550","loot":"Lâmina Abissal, Coração do Abismo, Coroa do Leviatã"},
        {"island":"golden_kingdom","name":"Reino Dourado","range":"551-600","theme":"Reino dourado flutuante","requirement":"Fase 550 + 8000 Gold + VIP 5","max_stage":max_stage,"vip":vip,"can_unlock":unlocked_abyss && max_stage>=550 && gold>=8000 && vip>=5,"unlocked":unlocked_gold,"island_max":island_max_gold,"mobs":["golden_golem","treasure_mimic","golden_phoenix"],"boss":"Rei Dourado 600","loot":"Lâmina Dourada, Armadura do Rei, Coroa Dourada Suprema"}
    ])))
}

async fn unlock(state: web::Data<Arc<AppState>>, user: AuthenticatedUser) -> AppResult<HttpResponse> {
    unlock_specific(state, user, web::Path::from("abyss_island".to_string())).await
}

async fn unlock_specific(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, island_code: web::Path<String>) -> AppResult<HttpResponse> {
    let code = island_code.into_inner();
    if code != "abyss_island" && code != "golden_kingdom" { return Err(AppError::Validation("ilha deve ser abyss_island ou golden_kingdom".into())); }
    let mut tx = state.db.begin().await?;
    let max_stage: i16 = sqlx::query_scalar("SELECT COALESCE(max_stage,0)::smallint FROM stage_progress WHERE user_id=$1 FOR UPDATE").bind(user.user_id).fetch_one(&mut *tx).await?;
    let gold: i64 = sqlx::query_scalar("SELECT gold FROM users WHERE id=$1 FOR UPDATE").bind(user.user_id).fetch_one(&mut *tx).await?;
    let vip: i16 = sqlx::query_scalar("SELECT vip_level FROM users WHERE id=$1").bind(user.user_id).fetch_one(&mut *tx).await.unwrap_or(0);
    let (need_stage, need_gold, need_vip, reward_skin) = if code=="abyss_island" { (500,5000,0,"wings_t8_island_abyss") } else { (550,8000,5,"wings_t8_golden_kingdom") };
    if code=="golden_kingdom" {
        let abyss_unlocked: bool = sqlx::query_scalar("SELECT COALESCE((SELECT unlocked FROM island_progress WHERE user_id=$1 AND island_code='abyss_island'), false)").bind(user.user_id).fetch_one(&mut *tx).await?;
        if !abyss_unlocked { return Err(AppError::Validation("complete Ilha 11 primeiro".into())); }
    }
    if max_stage < need_stage { return Err(AppError::Validation(format!("requer Fase {need_stage}"))); }
    if gold < need_gold { return Err(AppError::Validation(format!("{need_gold} Gold necessários"))); }
    if vip < need_vip { return Err(AppError::Validation(format!("VIP {need_vip} necessário"))); }
    let already: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM island_progress WHERE user_id=$1 AND island_code=$2 AND unlocked=true)").bind(user.user_id).bind(&code).fetch_one(&mut *tx).await?;
    if already { return Err(AppError::Validation("Ilha já desbloqueada".into())); }
    sqlx::query("UPDATE users SET gold=gold-$2 WHERE id=$1").bind(user.user_id).bind(need_gold as i64).execute(&mut *tx).await?;
    let start_stage = if code=="abyss_island" {501} else {551};
    sqlx::query("INSERT INTO island_progress (user_id, island_code, unlocked, unlocked_at, max_stage) VALUES ($1,$2,true,now(),$3) ON CONFLICT (user_id,island_code) DO UPDATE SET unlocked=true, unlocked_at=now()")
        .bind(user.user_id).bind(&code).bind(start_stage).execute(&mut *tx).await?;
    let skin: Option<Uuid> = sqlx::query_scalar("SELECT id FROM cosmetic_skins WHERE skin_code=$1").bind(reward_skin).fetch_optional(&mut *tx).await?;
    if let Some(sid)=skin { sqlx::query("INSERT INTO user_cosmetic_skins (user_id, skin_id) VALUES ($1,$2) ON CONFLICT DO NOTHING").bind(user.user_id).bind(sid).execute(&mut *tx).await?; }
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"unlocked":code,"stage":start_stage,"reward":reward_skin})))
}

async fn enter(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, stage: web::Path<u16>) -> AppResult<HttpResponse> {
    let s = stage.into_inner();
    if !(501..=600).contains(&s) { return Err(AppError::Validation("Ilhas 501-600 apenas".into())); }
    let code = if s<=550 {"abyss_island"} else {"golden_kingdom"};
    let unlocked: bool = sqlx::query_scalar("SELECT COALESCE((SELECT unlocked FROM island_progress WHERE user_id=$1 AND island_code=$2), false)").bind(user.user_id).bind(code).fetch_one(&state.db).await?;
    if !unlocked { return Err(AppError::Validation(format!("desbloqueie {code} primeiro"))); }
    let (enemies, boss) = if s<=550 { (vec!["abyssal_horror","deep_one"], s==550) } else { (vec!["golden_golem","treasure_mimic"], s==600) };
    let res: serde_json::Value = serde_json::json!({"stage":s,"island":code,"enemies":enemies,"boss":boss,"note":"Use POST /api/v1/combat/start com stage 501-600"});
    Ok(HttpResponse::Ok().json(res))
}
