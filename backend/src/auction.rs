use std::sync::Arc;
use actix_web::{web, HttpResponse};
use chrono::{Duration, Utc};
use serde::Deserialize;
use uuid::Uuid;

use crate::{auth::middleware::AuthenticatedUser, error::{AppError, AppResult}, state::AppState};

#[derive(Deserialize)]
struct CreateAuction { inventory_item_id: Uuid, start_price: i64, duration_hours: i64 } // 6,12,24,48
#[derive(Deserialize)]
struct BidRequest { amount: i64 }

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/auctions")
        .route("", web::get().to(list))
        .route("", web::post().to(create))
        .route("/{id}/bid", web::post().to(bid))
        .route("/{id}", web::get().to(get))
    );
}

async fn list(state: web::Data<Arc<AppState>>, _user: AuthenticatedUser) -> AppResult<HttpResponse> {
    let rows: Vec<(Uuid,Uuid,Uuid,i64,i64,chrono::DateTime<Utc>,String)> = sqlx::query_as("SELECT id, seller_user_id, inventory_item_id, start_price, current_price, ends_at, status FROM auctions WHERE status='active' ORDER BY ends_at ASC LIMIT 50")
        .fetch_all(&state.db).await?;
    Ok(HttpResponse::Ok().json(rows.into_iter().map(|(id,seller,item,start,cur,ends,status)| serde_json::json!({"id":id,"seller":seller,"item_id":item,"start_price":start,"current_price":cur,"ends_at":ends,"status":status})).collect::<Vec<_>>()))
}

async fn get(state: web::Data<Arc<AppState>>, _user: AuthenticatedUser, id: web::Path<Uuid>) -> AppResult<HttpResponse> {
    let row: Option<(Uuid,Uuid,i64,i64,Option<Uuid>,chrono::DateTime<Utc>,String)> = sqlx::query_as("SELECT id, inventory_item_id, start_price, current_price, current_winner, ends_at, status FROM auctions WHERE id=$1")
        .bind(id.into_inner()).fetch_optional(&state.db).await?;
    let (aid,item,start,cur,winner,ends,status)= row.ok_or(AppError::NotFound)?;
    let bids: Vec<(Uuid,i64,chrono::DateTime<Utc>)> = sqlx::query_as("SELECT bidder_user_id, amount, created_at FROM auction_bids WHERE auction_id=$1 ORDER BY amount DESC LIMIT 10").bind(aid).fetch_all(&state.db).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"id":aid,"item_id":item,"start_price":start,"current_price":cur,"winner":winner,"ends_at":ends,"status":status,"bids":bids.into_iter().map(|(u,a,t)| serde_json::json!({"bidder":u,"amount":a,"at":t})).collect::<Vec<_>>() })))
}

async fn create(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, body: web::Json<CreateAuction>) -> AppResult<HttpResponse> {
    if ![6,12,24,48].contains(&body.duration_hours) { return Err(AppError::Validation("duração 6/12/24/48h".into())); }
    let mut tx = state.db.begin().await?;
    let owner: Option<(Uuid,String,String)> = sqlx::query_as("SELECT i.user_id, t.rarity::text, t.code FROM inventory_items i JOIN item_templates t ON t.id=i.template_id WHERE i.id=$1 FOR UPDATE").bind(body.inventory_item_id).fetch_optional(&mut *tx).await?;
    let (owner_id, rarity, code) = owner.ok_or(AppError::NotFound)?;
    if owner_id != user.user_id { return Err(AppError::NotFound); }
    // Primordial só leilão 48h
    if rarity=="primordial" && body.duration_hours != 48 { return Err(AppError::Validation("Primordial exige leilão 48h".into())); }
    if !["legendary","mythic","divine","primordial"].contains(&rarity.as_str()) { return Err(AppError::Validation("apenas Lendário+ pode ir a leilão".into())); }
    let equipped: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM equipment_slots WHERE inventory_item_id=$1)").bind(body.inventory_item_id).fetch_one(&mut *tx).await?;
    if equipped { return Err(AppError::Validation("desequipe antes".into())); }
    let ends = Utc::now() + Duration::hours(body.duration_hours);
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO auctions (id, seller_user_id, inventory_item_id, start_price, current_price, ends_at) VALUES ($1,$2,$3,$4,$4,$5)")
        .bind(id).bind(user.user_id).bind(body.inventory_item_id).bind(body.start_price).bind(ends).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(HttpResponse::Created().json(serde_json::json!({"auction_id":id,"ends_at":ends,"note": if code=="primordial_crown_primordial" {"Primordial sempre leilão"} else {""}})))
}

async fn bid(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, id: web::Path<Uuid>, body: web::Json<BidRequest>) -> AppResult<HttpResponse> {
    let auction_id=id.into_inner();
    let mut tx = state.db.begin().await?;
    let row: Option<(i64,i64,Option<Uuid>,chrono::DateTime<Utc>,String,Uuid)> = sqlx::query_as("SELECT start_price, current_price, current_winner, ends_at, status, seller_user_id FROM auctions WHERE id=$1 FOR UPDATE").bind(auction_id).fetch_optional(&mut *tx).await?;
    let (start, cur, winner, mut ends, status, seller)= row.ok_or(AppError::NotFound)?;
    if status!="active" { return Err(AppError::Validation("leilão não ativo".into())); }
    if seller==user.user_id { return Err(AppError::Validation("não pode dar lance no próprio leilão".into())); }
    if Utc::now() > ends { return Err(AppError::Validation("leilão expirado".into())); }
    if body.amount <= cur { return Err(AppError::Validation(format!("lance deve superar {cur}"))); }
    let diamonds: i64 = sqlx::query_scalar("SELECT diamonds FROM users WHERE id=$1 FOR UPDATE").bind(user.user_id).fetch_one(&mut *tx).await?;
    if diamonds < body.amount { return Err(AppError::Validation("diamantes insuficientes".into())); }
    // Anti-snipe: se <30min para fim, estende 30min
    if (ends - Utc::now()).num_minutes() < 30 {
        ends = Utc::now() + Duration::minutes(30);
        sqlx::query("UPDATE auctions SET ends_at=$2 WHERE id=$1").bind(auction_id).bind(ends).execute(&mut *tx).await?;
    }
    // Reserva diamantes do novo vencedor, libera anterior (simplificado: não reservar, só validar)
    sqlx::query("UPDATE auctions SET current_price=$2, current_winner=$3, ends_at=$4 WHERE id=$1").bind(auction_id).bind(body.amount).bind(user.user_id).bind(ends).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO auction_bids (auction_id, bidder_user_id, amount) VALUES ($1,$2,$3)").bind(auction_id).bind(user.user_id).bind(body.amount).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"bid":body.amount,"ends_at":ends,"anti_snipe": (ends - Utc::now()).num_minutes() < 31 })))
}
