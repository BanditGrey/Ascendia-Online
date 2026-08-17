use std::sync::Arc;
use actix_web::{web, HttpResponse};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{auth::middleware::AuthenticatedUser, error::{AppError, AppResult}, state::AppState};

#[derive(Deserialize)]
struct StartExpedition { character_id: Uuid, duration: String } // 2h,4h,8h,12h,24h

#[derive(Serialize)]
struct ExpeditionView { id: Uuid, character_id: Uuid, duration: String, ends_at: chrono::DateTime<Utc>, claimed: bool }

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/expeditions")
        .route("", web::get().to(list))
        .route("/start", web::post().to(start))
        .route("/{id}/claim", web::post().to(claim))
    );
}

async fn list(state: web::Data<Arc<AppState>>, user: AuthenticatedUser) -> AppResult<HttpResponse> {
    let rows: Vec<(Uuid,Uuid,String,chrono::DateTime<Utc>,bool)> = sqlx::query_as("SELECT id, character_id, duration::text, ends_at, claimed FROM expeditions WHERE user_id=$1 ORDER BY started_at DESC LIMIT 20")
        .bind(user.user_id).fetch_all(&state.db).await?;
    let out: Vec<ExpeditionView> = rows.into_iter().map(|(id,cid,dur,ends,claimed)| ExpeditionView{ id, character_id:cid, duration:dur, ends_at:ends, claimed }).collect();
    Ok(HttpResponse::Ok().json(out))
}

async fn start(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, body: web::Json<StartExpedition>) -> AppResult<HttpResponse> {
    let dur = body.duration.as_str();
    if !["2h","4h","8h","12h","24h"].contains(&dur) { return Err(AppError::Validation("duração deve ser 2h,4h,8h,12h,24h".into())); }
    // Verifica character não está no squad e pertence ao user
    let owns: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM characters WHERE id=$1 AND user_id=$2)").bind(body.character_id).bind(user.user_id).fetch_one(&state.db).await?;
    if !owns { return Err(AppError::NotFound); }
    let in_squad: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM squad_slots ss JOIN squads s ON s.id=ss.squad_id WHERE ss.character_id=$1 AND s.user_id=$2 AND s.is_active=true)").bind(body.character_id).bind(user.user_id).fetch_one(&state.db).await?;
    if in_squad { return Err(AppError::Validation("personagem no squad ativo não pode expedir".into())); }
    // Slots: 3 base, VIP até 8
    let vip: i16 = sqlx::query_scalar("SELECT vip_level FROM users WHERE id=$1").bind(user.user_id).fetch_one(&state.db).await.unwrap_or(0);
    let max_slots = if vip >= 8 { 8 } else if vip >= 5 { 5 } else { 3 };
    let active: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM expeditions WHERE user_id=$1 AND claimed=false AND ends_at > now()").bind(user.user_id).fetch_one(&state.db).await?;
    if active >= max_slots as i64 { return Err(AppError::Validation(format!("slots de expedição cheios ({max_slots})"))); }
    let already: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM expeditions WHERE character_id=$1 AND claimed=false AND ends_at > now())").bind(body.character_id).fetch_one(&state.db).await?;
    if already { return Err(AppError::Validation("personagem já em expedição".into())); }
    let hours = match dur { "2h"=>2, "4h"=>4, "8h"=>8, "12h"=>12, "24h"=>24, _=>2 };
    let ends = Utc::now() + Duration::hours(hours as i64);
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO expeditions (id, user_id, character_id, duration, ends_at) VALUES ($1,$2,$3,$4::text::expedition_duration,$5)")
        .bind(id).bind(user.user_id).bind(body.character_id).bind(dur).bind(ends).execute(&state.db).await?;
    Ok(HttpResponse::Created().json(serde_json::json!({"expedition_id":id,"ends_at":ends})))
}

async fn claim(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, id: web::Path<Uuid>) -> AppResult<HttpResponse> {
    let eid = id.into_inner();
    let mut tx = state.db.begin().await?;
    let row: Option<(Uuid, chrono::DateTime<Utc>, bool)> = sqlx::query_as("SELECT character_id, ends_at, claimed FROM expeditions WHERE id=$1 AND user_id=$2 FOR UPDATE")
        .bind(eid).bind(user.user_id).fetch_optional(&mut *tx).await?;
    let (cid, ends, claimed) = row.ok_or(AppError::NotFound)?;
    if claimed { return Err(AppError::Validation("já coletado".into())); }
    if Utc::now() < ends { return Err(AppError::Validation("expedição ainda em andamento".into())); }
    // Recompensas por duração
    let (gold, frags) = match (ends - (ends - Duration::hours(2))).num_hours() { // dummy, use duration enum
        _ => (100, 3) // simplificado: mapeia via query
    };
    // Busca duração real
    let dur: String = sqlx::query_scalar("SELECT duration::text FROM expeditions WHERE id=$1").bind(eid).fetch_one(&mut *tx).await?;
    let (gold, frags) = match dur.as_str() {
        "2h" => (80, 2),
        "4h" => (180, 4),
        "8h" => (400, 8),
        "12h" => (650, 12),
        "24h" => (1400, 25),
        _ => (100, 3),
    };
    sqlx::query("UPDATE users SET gold=gold+$2 WHERE id=$1").bind(user.user_id).bind(gold).execute(&mut *tx).await?;
    for kind in ["wings","mount","pet","aura"] {
        sqlx::query("INSERT INTO cosmetic_progress (user_id,cosmetic_type,fragments) VALUES ($1,$2,$3) ON CONFLICT (user_id,cosmetic_type) DO UPDATE SET fragments=cosmetic_progress.fragments+EXCLUDED.fragments")
            .bind(user.user_id).bind(kind).bind(frags).execute(&mut *tx).await?;
    }
    sqlx::query("UPDATE expeditions SET claimed=true WHERE id=$1").bind(eid).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"claimed":eid,"gold":gold,"frags":frags})))
}
