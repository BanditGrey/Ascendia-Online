use std::sync::Arc;
use actix_web::{web, HttpResponse};
use chrono::Utc;
use uuid::Uuid;

use crate::{auth::middleware::AuthenticatedUser, error::{AppError, AppResult}, state::AppState};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/quests")
        .route("/daily", web::get().to(daily))
        .route("/daily/claim", web::post().to(claim_daily))
        .route("/weekly", web::get().to(weekly))
        .route("/weekly/claim", web::post().to(claim_weekly))
        .route("/achievements", web::get().to(achievements))
    );
}

async fn daily(state: web::Data<Arc<AppState>>, user: AuthenticatedUser) -> AppResult<HttpResponse> {
    ensure_daily_seeded(&state, user.user_id).await?;
    let rows: Vec<(String,i32,i32,bool)> = sqlx::query_as("SELECT quest_code, progress, target, claimed FROM daily_quests WHERE user_id=$1 AND quest_date=CURRENT_DATE ORDER BY quest_code")
        .bind(user.user_id).fetch_all(&state.db).await?;
    Ok(HttpResponse::Ok().json(rows.into_iter().map(|(c,p,t,cl)| serde_json::json!({"code":c,"progress":p,"target":t,"claimed":cl})).collect::<Vec<_>>()))
}

async fn weekly(state: web::Data<Arc<AppState>>, user: AuthenticatedUser) -> AppResult<HttpResponse> {
    ensure_weekly_seeded(&state, user.user_id).await?;
    let rows: Vec<(String,i32,i32,bool)> = sqlx::query_as("SELECT quest_code, progress, target, claimed FROM weekly_quests WHERE user_id=$1 AND week_start=date_trunc('week', now())::date ORDER BY quest_code")
        .bind(user.user_id).fetch_all(&state.db).await?;
    Ok(HttpResponse::Ok().json(rows.into_iter().map(|(c,p,t,cl)| serde_json::json!({"code":c,"progress":p,"target":t,"claimed":cl})).collect::<Vec<_>>()))
}

async fn achievements(state: web::Data<Arc<AppState>>, user: AuthenticatedUser) -> AppResult<HttpResponse> {
    let rows: Vec<(String,String,String,i32,i32,bool)> = sqlx::query_as("SELECT a.code, a.category, a.name, a.target, COALESCE(pa.progress,0), COALESCE(pa.claimed,false) FROM achievements a LEFT JOIN player_achievements pa ON pa.code=a.code AND pa.user_id=$1 ORDER BY a.category, a.code")
        .bind(user.user_id).fetch_all(&state.db).await?;
    Ok(HttpResponse::Ok().json(rows.into_iter().map(|(code,cat,name,target,prog,claimed)| serde_json::json!({"code":code,"category":cat,"name":name,"target":target,"progress":prog,"claimed":claimed})).collect::<Vec<_>>()))
}

#[derive(serde::Deserialize)]
struct ClaimBody { quest_code: String }

async fn claim_daily(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, body: web::Json<ClaimBody>) -> AppResult<HttpResponse> {
    let mut tx = state.db.begin().await?;
    let row: Option<(i32,i32,bool)> = sqlx::query_as("SELECT progress, target, claimed FROM daily_quests WHERE user_id=$1 AND quest_code=$2 AND quest_date=CURRENT_DATE FOR UPDATE")
        .bind(user.user_id).bind(&body.quest_code).fetch_optional(&mut *tx).await?;
    let (prog,target,claimed)= row.ok_or(AppError::NotFound)?;
    if claimed { return Err(AppError::Validation("já coletado".into())); }
    if prog < target { return Err(AppError::Validation("progresso insuficiente".into())); }
    sqlx::query("UPDATE daily_quests SET claimed=true WHERE user_id=$1 AND quest_code=$2 AND quest_date=CURRENT_DATE").bind(user.user_id).bind(&body.quest_code).execute(&mut *tx).await?;
    sqlx::query("UPDATE users SET gold=gold+100, diamonds=diamonds+5 WHERE id=$1").bind(user.user_id).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"claimed":body.quest_code,"gold":100,"diamonds":5})))
}

async fn claim_weekly(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, body: web::Json<ClaimBody>) -> AppResult<HttpResponse> {
    let mut tx = state.db.begin().await?;
    let row: Option<(i32,i32,bool)> = sqlx::query_as("SELECT progress, target, claimed FROM weekly_quests WHERE user_id=$1 AND quest_code=$2 AND week_start=date_trunc('week', now())::date FOR UPDATE")
        .bind(user.user_id).bind(&body.quest_code).fetch_optional(&mut *tx).await?;
    let (prog,target,claimed)= row.ok_or(AppError::NotFound)?;
    if claimed { return Err(AppError::Validation("já coletado".into())); }
    if prog < target { return Err(AppError::Validation("progresso insuficiente".into())); }
    sqlx::query("UPDATE weekly_quests SET claimed=true WHERE user_id=$1 AND quest_code=$2 AND week_start=date_trunc('week', now())::date").bind(user.user_id).bind(&body.quest_code).execute(&mut *tx).await?;
    sqlx::query("UPDATE users SET gold=gold+500, diamonds=diamonds+20 WHERE id=$1").bind(user.user_id).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"claimed":body.quest_code,"gold":500,"diamonds":20})))
}

async fn ensure_daily_seeded(state: &AppState, user_id: Uuid) -> AppResult<()> {
    let codes = ["complete_stages","arena_fights","dungeon_clears","enhance_items","kill_mobs"];
    for code in codes {
        sqlx::query("INSERT INTO daily_quests (user_id, quest_code, target) VALUES ($1,$2,3) ON CONFLICT DO NOTHING").bind(user_id).bind(code).execute(&state.db).await?;
    }
    Ok(())
}
async fn ensure_weekly_seeded(state: &AppState, user_id: Uuid) -> AppResult<()> {
    let codes = ["raid_boss","arena_wins","tower_floor","chapter_hard","market_trades"];
    for code in codes {
        sqlx::query("INSERT INTO weekly_quests (user_id, quest_code, target) VALUES ($1,$2,5) ON CONFLICT DO NOTHING").bind(user_id).bind(code).execute(&state.db).await?;
    }
    Ok(())
}
