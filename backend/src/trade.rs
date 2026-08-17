use std::sync::Arc;
use actix_web::{web, HttpResponse};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{auth::middleware::AuthenticatedUser, error::{AppError, AppResult}, state::AppState};

#[derive(Deserialize)]
struct CreateTrade { to_user_id: Uuid, offer_item_ids: Vec<Uuid>, request_item_ids: Vec<Uuid>, offer_diamonds: i64, request_diamonds: i64 }

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/trades")
        .route("", web::get().to(list))
        .route("", web::post().to(create))
        .route("/{id}/accept", web::post().to(accept))
        .route("/{id}/cancel", web::post().to(cancel))
    );
}

async fn list(state: web::Data<Arc<AppState>>, user: AuthenticatedUser) -> AppResult<HttpResponse> {
    let trades: Vec<(Uuid,Uuid,Uuid,String)> = sqlx::query_as("SELECT id, from_user_id, to_user_id, status FROM trades WHERE from_user_id=$1 OR to_user_id=$1 ORDER BY created_at DESC LIMIT 20")
        .bind(user.user_id).fetch_all(&state.db).await?;
    Ok(HttpResponse::Ok().json(trades.into_iter().map(|(id,from,to,status)| serde_json::json!({"id":id,"from":from,"to":to,"status":status})).collect::<Vec<_>>()))
}

async fn create(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, body: web::Json<CreateTrade>) -> AppResult<HttpResponse> {
    if user.user_id == body.to_user_id { return Err(AppError::Validation("não pode negociar consigo mesmo".into())); }
    if body.offer_item_ids.is_empty() && body.offer_diamonds==0 { return Err(AppError::Validation("oferta vazia".into())); }
    let mut tx = state.db.begin().await?;
    // Valida ownership dos itens oferecidos
    for item_id in &body.offer_item_ids {
        let owner: Option<Uuid> = sqlx::query_scalar("SELECT user_id FROM inventory_items WHERE id=$1 FOR UPDATE").bind(item_id).fetch_optional(&mut *tx).await?;
        if owner.ok_or(AppError::NotFound)? != user.user_id { return Err(AppError::Validation("item não pertence a você".into())); }
        let bound: bool = sqlx::query_scalar("SELECT bound FROM inventory_items WHERE id=$1").bind(item_id).fetch_one(&mut *tx).await?;
        if bound { return Err(AppError::Validation("item bound não pode ser trocado".into())); }
    }
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO trades (id, from_user_id, to_user_id, from_diamonds, to_diamonds) VALUES ($1,$2,$3,$4,$5)")
        .bind(id).bind(user.user_id).bind(body.to_user_id).bind(body.offer_diamonds).bind(body.request_diamonds).execute(&mut *tx).await?;
    for item_id in &body.offer_item_ids {
        sqlx::query("INSERT INTO trade_items (trade_id, user_id, inventory_item_id) VALUES ($1,$2,$3)").bind(id).bind(user.user_id).bind(item_id).execute(&mut *tx).await?;
    }
    for item_id in &body.request_item_ids {
        // Apenas registra intenção, não valida ownership do outro ainda
        sqlx::query("INSERT INTO trade_items (trade_id, user_id, inventory_item_id) VALUES ($1,$2,$3)").bind(id).bind(body.to_user_id).bind(item_id).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    Ok(HttpResponse::Created().json(serde_json::json!({"trade_id":id,"expires_in":60})))
}

async fn accept(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, id: web::Path<Uuid>) -> AppResult<HttpResponse> {
    let trade_id = id.into_inner();
    let mut tx = state.db.begin().await?;
    let row: Option<(Uuid,Uuid,i64,i64,String,chrono::DateTime<chrono::Utc>)> = sqlx::query_as("SELECT from_user_id, to_user_id, from_diamonds, to_diamonds, status, expires_at FROM trades WHERE id=$1 FOR UPDATE").bind(trade_id).fetch_optional(&mut *tx).await?;
    let (from,to,from_d,to_d,status,expires)= row.ok_or(AppError::NotFound)?;
    if status!="pending" { return Err(AppError::Validation("trade não pendente".into())); }
    if Utc::now() > expires { 
        sqlx::query("UPDATE trades SET status='cancelled' WHERE id=$1").bind(trade_id).execute(&mut *tx).await?;
        return Err(AppError::Validation("trade expirou (60s anti-scam)".into()));
    }
    if user.user_id != to { return Err(AppError::Unauthorized); }
    // Verifica diamantes
    let from_diam: i64 = sqlx::query_scalar("SELECT diamonds FROM users WHERE id=$1 FOR UPDATE").bind(from).fetch_one(&mut *tx).await?;
    let to_diam: i64 = sqlx::query_scalar("SELECT diamonds FROM users WHERE id=$1 FOR UPDATE").bind(to).fetch_one(&mut *tx).await?;
    if from_diam < from_d { return Err(AppError::Validation("oferente sem diamantes".into())); }
    if to_diam < to_d { return Err(AppError::Validation("você sem diamantes".into())); }
    // Transfere diamantes
    if from_d>0 { sqlx::query("UPDATE users SET diamonds=diamonds-$2 WHERE id=$1").bind(from).bind(from_d).execute(&mut *tx).await?; sqlx::query("UPDATE users SET diamonds=diamonds+$2 WHERE id=$1").bind(to).bind(from_d).execute(&mut *tx).await?; }
    if to_d>0 { sqlx::query("UPDATE users SET diamonds=diamonds-$2 WHERE id=$1").bind(to).bind(to_d).execute(&mut *tx).await?; sqlx::query("UPDATE users SET diamonds=diamonds+$2 WHERE id=$1").bind(from).bind(to_d).execute(&mut *tx).await?; }
    // Transfere itens (cada lado recebe do outro)
    let items: Vec<(Uuid,Uuid)> = sqlx::query_as("SELECT user_id, inventory_item_id FROM trade_items WHERE trade_id=$1").bind(trade_id).fetch_all(&mut *tx).await?;
    for (owner, item_id) in items {
        let new_owner = if owner==from { to } else { from };
        sqlx::query("UPDATE inventory_items SET user_id=$2 WHERE id=$1").bind(item_id).bind(new_owner).execute(&mut *tx).await?;
        // Remove de equipment se equipado
        sqlx::query("DELETE FROM equipment_slots WHERE inventory_item_id=$1").bind(item_id).execute(&mut *tx).await?;
    }
    sqlx::query("UPDATE trades SET status='completed' WHERE id=$1").bind(trade_id).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"completed":trade_id})))
}

async fn cancel(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, id: web::Path<Uuid>) -> AppResult<HttpResponse> {
    let trade_id=id.into_inner();
    let owner: Option<Uuid> = sqlx::query_scalar("SELECT from_user_id FROM trades WHERE id=$1 AND status='pending'").bind(trade_id).fetch_optional(&state.db).await?;
    if owner.ok_or(AppError::NotFound)? != user.user_id { return Err(AppError::Unauthorized); }
    sqlx::query("UPDATE trades SET status='cancelled' WHERE id=$1").bind(trade_id).execute(&state.db).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"cancelled":trade_id})))
}
