use std::sync::Arc;
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{auth::middleware::AuthenticatedUser, error::{AppError, AppResult}, inventory::stats::recalculate, state::AppState};

#[derive(FromRow, Serialize)]
struct Rune { id: Uuid, code: String, rune_type: String, bonus: serde_json::Value }

#[derive(Deserialize)]
struct SocketRequest { inventory_item_id: Uuid, socket_index: i16, rune_id: Uuid }
#[derive(Deserialize)]
struct UnsocketRequest { inventory_item_id: Uuid, socket_index: i16 }

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/runes")
        .route("", web::get().to(list_runes))
        .route("/socket", web::post().to(socket))
        .route("/unsocket", web::post().to(unsocket))
    );
}

async fn list_runes(state: web::Data<Arc<AppState>>, _user: AuthenticatedUser) -> AppResult<HttpResponse> {
    let runes: Vec<Rune> = sqlx::query_as("SELECT id, code, rune_type, bonus FROM runes ORDER BY code").fetch_all(&state.db).await?;
    Ok(HttpResponse::Ok().json(runes))
}

async fn socket(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, body: web::Json<SocketRequest>) -> AppResult<HttpResponse> {
    if !(1..=4).contains(&body.socket_index) { return Err(AppError::Validation("socket_index 1-4".into())); }
    let mut tx = state.db.begin().await?;
    let owner: Option<Uuid> = sqlx::query_scalar("SELECT user_id FROM inventory_items WHERE id=$1 FOR UPDATE").bind(body.inventory_item_id).fetch_optional(&mut *tx).await?;
    if owner.ok_or(AppError::NotFound)? != user.user_id { return Err(AppError::NotFound); }
    // Verifica raridade Épico+ tem sockets
    let rarity: String = sqlx::query_scalar("SELECT t.rarity::text FROM inventory_items i JOIN item_templates t ON t.id=i.template_id WHERE i.id=$1").bind(body.inventory_item_id).fetch_one(&mut *tx).await?;
    if !["epic","legendary","mythic","divine","primordial"].contains(&rarity.as_str()) { return Err(AppError::Validation("apenas Épico+ tem sockets".into())); }
    let rune_exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM runes WHERE id=$1)").bind(body.rune_id).fetch_one(&mut *tx).await?;
    if !rune_exists { return Err(AppError::NotFound); }
    sqlx::query("INSERT INTO item_sockets (inventory_item_id, socket_index, rune_id) VALUES ($1,$2,$3) ON CONFLICT (inventory_item_id, socket_index) DO UPDATE SET rune_id=EXCLUDED.rune_id")
        .bind(body.inventory_item_id).bind(body.socket_index).bind(body.rune_id).execute(&mut *tx).await?;
    // Recalcula se equipado
    if let Some(char_id) = sqlx::query_scalar::<_,Uuid>("SELECT character_id FROM equipment_slots WHERE inventory_item_id=$1").bind(body.inventory_item_id).fetch_optional(&mut *tx).await? {
        recalculate(&mut tx, user.user_id, char_id).await?;
    }
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"socketed":body.socket_index})))
}

async fn unsocket(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, body: web::Json<UnsocketRequest>) -> AppResult<HttpResponse> {
    let mut tx = state.db.begin().await?;
    let owner: Option<Uuid> = sqlx::query_scalar("SELECT user_id FROM inventory_items WHERE id=$1").bind(body.inventory_item_id).fetch_optional(&mut *tx).await?;
    if owner.ok_or(AppError::NotFound)? != user.user_id { return Err(AppError::NotFound); }
    sqlx::query("UPDATE item_sockets SET rune_id=NULL WHERE inventory_item_id=$1 AND socket_index=$2").bind(body.inventory_item_id).bind(body.socket_index).execute(&mut *tx).await?;
    if let Some(char_id) = sqlx::query_scalar::<_,Uuid>("SELECT character_id FROM equipment_slots WHERE inventory_item_id=$1").bind(body.inventory_item_id).fetch_optional(&mut *tx).await? {
        recalculate(&mut tx, user.user_id, char_id).await?;
    }
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"unsocketed":body.socket_index})))
}
