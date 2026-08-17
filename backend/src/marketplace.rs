use std::sync::Arc;

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{auth::middleware::AuthenticatedUser, error::{AppError, AppResult}, state::AppState};

#[derive(FromRow, Serialize)]
struct ListingView { id: Uuid, seller_user_id: Uuid, inventory_item_id: Uuid, price_diamonds: i64, status: String, listed_at: chrono::DateTime<chrono::Utc>, item_name: String, rarity: String }

#[derive(Deserialize)]
struct CreateListing { inventory_item_id: Uuid, price_diamonds: i64 }

#[derive(Deserialize)]
struct MarketQuery { #[serde(default)] offset: i64, #[serde(default="default_limit")] limit: i64 }
fn default_limit() -> i64 { 20 }

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/marketplace")
        .route("", web::get().to(list))
        .route("", web::post().to(create))
        .route("/{id}/buy", web::post().to(buy))
        .route("/{id}", web::delete().to(cancel))
    );
}

async fn list(state: web::Data<Arc<AppState>>, _user: AuthenticatedUser, q: web::Query<MarketQuery>) -> AppResult<HttpResponse> {
    let limit = q.limit.clamp(1, 50);
    let offset = q.offset.max(0);
    let listings: Vec<ListingView> = sqlx::query_as("SELECT l.id,l.seller_user_id,l.inventory_item_id,l.price_diamonds,l.status,l.listed_at,t.name as item_name,t.rarity::text as rarity FROM marketplace_listings l JOIN inventory_items i ON i.id=l.inventory_item_id JOIN item_templates t ON t.id=i.template_id WHERE l.status='active' ORDER BY l.listed_at DESC LIMIT $1 OFFSET $2")
        .bind(limit).bind(offset).fetch_all(&state.db).await?;
    Ok(HttpResponse::Ok().json(listings))
}

async fn create(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, body: web::Json<CreateListing>) -> AppResult<HttpResponse> {
    if body.price_diamonds <= 0 { return Err(AppError::Validation("preço deve ser positivo".into())); }
    let mut tx = state.db.begin().await?;
    // Verifica ownership e se não está equipado e não é bound VIP
    let row: Option<(Uuid, bool, Option<chrono::DateTime<chrono::Utc>>)> = sqlx::query_as("SELECT user_id,bound,trade_locked_until FROM inventory_items WHERE id=$1 FOR UPDATE")
        .bind(body.inventory_item_id).fetch_optional(&mut *tx).await?;
    let (owner, bound, lock) = row.ok_or(AppError::NotFound)?;
    if owner != user.user_id { return Err(AppError::NotFound); }
    if bound { return Err(AppError::Validation("item bound não pode ser vendido".into())); }
    if let Some(until) = lock { if until > chrono::Utc::now() { return Err(AppError::Validation("trade lock ativo 24h".into())); } }
    let equipped: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM equipment_slots WHERE inventory_item_id=$1)").bind(body.inventory_item_id).fetch_one(&mut *tx).await?;
    if equipped { return Err(AppError::Validation("desequipe antes de vender".into())); }
    let active: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM marketplace_listings WHERE seller_user_id=$1 AND status='active'").bind(user.user_id).fetch_one(&mut *tx).await?;
    let max = if { let vip: i16 = sqlx::query_scalar("SELECT vip_level FROM users WHERE id=$1").bind(user.user_id).fetch_one(&mut *tx).await?; vip } >= 5 { 50 } else { 20 };
    if active >= max { return Err(AppError::Validation(format!("limite de listagens ativas atingido ({max})"))); }
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO marketplace_listings (id,seller_user_id,inventory_item_id,price_diamonds) VALUES ($1,$2,$3,$4)")
        .bind(id).bind(user.user_id).bind(body.inventory_item_id).bind(body.price_diamonds).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(HttpResponse::Created().json(serde_json::json!({"listing_id":id})))
}

async fn buy(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, id: web::Path<Uuid>) -> AppResult<HttpResponse> {
    let listing_id = id.into_inner();
    let mut tx = state.db.begin().await?;
    let row: Option<(Uuid, Uuid, i64)> = sqlx::query_as("SELECT seller_user_id,inventory_item_id,price_diamonds FROM marketplace_listings WHERE id=$1 AND status='active' FOR UPDATE")
        .bind(listing_id).fetch_optional(&mut *tx).await?;
    let (seller, item_id, price) = row.ok_or(AppError::NotFound)?;
    if seller == user.user_id { return Err(AppError::Validation("não pode comprar próprio item".into())); }
    let buyer_diamonds: i64 = sqlx::query_scalar("SELECT diamonds FROM users WHERE id=$1 FOR UPDATE").bind(user.user_id).fetch_one(&mut *tx).await?;
    if buyer_diamonds < price { return Err(AppError::Validation("diamantes insuficientes".into())); }
    // Taxa 10% para o servidor (burn)
    let fee = price / 10;
    let seller_gain = price - fee;
    sqlx::query("UPDATE users SET diamonds=diamonds-$2 WHERE id=$1").bind(user.user_id).bind(price).execute(&mut *tx).await?;
    sqlx::query("UPDATE users SET diamonds=diamonds+$2 WHERE id=$1").bind(seller).bind(seller_gain).execute(&mut *tx).await?;
    sqlx::query("UPDATE inventory_items SET user_id=$2 WHERE id=$1").bind(item_id).bind(user.user_id).execute(&mut *tx).await?;
    sqlx::query("UPDATE marketplace_listings SET status='sold', sold_at=now() WHERE id=$1").bind(listing_id).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO audit_logs (actor_user_id,action,metadata) VALUES ($1,'MARKET_BOUGHT',$2)").bind(user.user_id).bind(serde_json::json!({"listing_id":listing_id,"price":price,"fee":fee})).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"bought":listing_id,"price":price,"fee":fee})))
}

async fn cancel(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, id: web::Path<Uuid>) -> AppResult<HttpResponse> {
    let listing_id = id.into_inner();
    let mut tx = state.db.begin().await?;
    let seller: Option<Uuid> = sqlx::query_scalar("SELECT seller_user_id FROM marketplace_listings WHERE id=$1 AND status='active' FOR UPDATE").bind(listing_id).fetch_optional(&mut *tx).await?;
    if seller.ok_or(AppError::NotFound)? != user.user_id { return Err(AppError::Unauthorized); }
    sqlx::query("UPDATE marketplace_listings SET status='cancelled' WHERE id=$1").bind(listing_id).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(HttpResponse::NoContent().finish())
}
