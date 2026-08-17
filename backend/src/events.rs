use std::sync::Arc;
use actix_web::{web, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::{auth::middleware::AuthenticatedUser, error::{AppError, AppResult}, state::AppState};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/events")
        .route("", web::get().to(list))
        .route("/{event_id}/progress", web::get().to(progress))
        .route("/{event_id}/claim", web::post().to(claim))
        .route("/shop/{event_id}", web::get().to(shop))
        .route("/shop/buy", web::post().to(buy))
    );
}

async fn list(state: web::Data<Arc<AppState>>, _user: AuthenticatedUser) -> AppResult<HttpResponse> {
    let rows: Vec<(Uuid,String,String,String,chrono::DateTime<chrono::Utc>,chrono::DateTime<chrono::Utc>,bool)> = sqlx::query_as("SELECT id, code, name, currency, starts_at, ends_at, is_active FROM seasonal_events WHERE is_active=true ORDER BY starts_at DESC").fetch_all(&state.db).await?;
    Ok(HttpResponse::Ok().json(rows.into_iter().map(|(id,code,name,curr,starts,ends,active)| serde_json::json!({"id":id,"code":code,"name":name,"currency":curr,"starts_at":starts,"ends_at":ends,"is_active":active})).collect::<Vec<_>>()))
}

async fn progress(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, event_id: web::Path<Uuid>) -> AppResult<HttpResponse> {
    let eid = event_id.into_inner();
    let amount: i64 = sqlx::query_scalar("SELECT COALESCE(currency_amount,0)::bigint FROM event_progress WHERE user_id=$1 AND event_id=$2").bind(user.user_id).bind(eid).fetch_optional(&state.db).await?.unwrap_or(0);
    // Simula ganho: a cada chamada, +10 se ainda ativo (demo)
    sqlx::query("INSERT INTO event_progress (user_id, event_id, currency_amount) VALUES ($1,$2,10) ON CONFLICT (user_id, event_id) DO UPDATE SET currency_amount=event_progress.currency_amount+10, updated_at=now()")
        .bind(user.user_id).bind(eid).execute(&state.db).await?;
    let new_amount: i64 = sqlx::query_scalar("SELECT currency_amount FROM event_progress WHERE user_id=$1 AND event_id=$2").bind(user.user_id).bind(eid).fetch_one(&state.db).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"event_id":eid,"currency_amount":new_amount,"gained":10})))
}

async fn claim(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, event_id: web::Path<Uuid>) -> AppResult<HttpResponse> {
    let eid = event_id.into_inner();
    let amount: i64 = sqlx::query_scalar("SELECT currency_amount FROM event_progress WHERE user_id=$1 AND event_id=$2").bind(user.user_id).bind(eid).fetch_optional(&state.db).await?.unwrap_or(0);
    if amount < 100 { return Err(AppError::Validation("precisa 100 moedas do evento".into())); }
    sqlx::query("UPDATE event_progress SET currency_amount=currency_amount-100 WHERE user_id=$1 AND event_id=$2").bind(user.user_id).bind(eid).execute(&state.db).await?;
    // Recompensa: 1 skin aleatória
    let skin: Option<Uuid> = sqlx::query_scalar("SELECT id FROM cosmetic_skins ORDER BY RANDOM() LIMIT 1").fetch_optional(&state.db).await?;
    if let Some(sid) = skin {
        sqlx::query("INSERT INTO user_cosmetic_skins (user_id, skin_id) VALUES ($1,$2) ON CONFLICT DO NOTHING").bind(user.user_id).bind(sid).execute(&state.db).await?;
    }
    Ok(HttpResponse::Ok().json(serde_json::json!({"claimed":eid,"cost":100,"reward":"skin"})))
}

async fn shop(state: web::Data<Arc<AppState>>, _user: AuthenticatedUser, event_id: web::Path<Uuid>) -> AppResult<HttpResponse> {
    let eid = event_id.into_inner();
    let items: Vec<(Uuid,String,i64)> = sqlx::query_as("SELECT id, item_code, cost FROM event_shop_items WHERE event_id=$1").bind(eid).fetch_all(&state.db).await?;
    Ok(HttpResponse::Ok().json(items.into_iter().map(|(id,code,cost)| serde_json::json!({"id":id,"item_code":code,"cost":cost})).collect::<Vec<_>>()))
}

#[derive(Deserialize)]
struct BuyRequest { shop_item_id: Uuid }

async fn buy(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, body: web::Json<BuyRequest>) -> AppResult<HttpResponse> {
    let mut tx = state.db.begin().await?;
    let row: Option<(Uuid,i64,Uuid)> = sqlx::query_as("SELECT event_id, cost, id FROM event_shop_items WHERE id=$1").bind(body.shop_item_id).fetch_optional(&mut *tx).await?;
    let (event_id, cost, _) = row.ok_or(AppError::NotFound)?;
    let balance: i64 = sqlx::query_scalar("SELECT COALESCE(currency_amount,0)::bigint FROM event_progress WHERE user_id=$1 AND event_id=$2").bind(user.user_id).bind(event_id).fetch_optional(&mut *tx).await?.unwrap_or(0);
    if balance < cost { return Err(AppError::Validation("moeda do evento insuficiente".into())); }
    sqlx::query("UPDATE event_progress SET currency_amount=currency_amount-$3 WHERE user_id=$1 AND event_id=$2").bind(user.user_id).bind(event_id).bind(cost).execute(&mut *tx).await?;
    // Entrega item (simula)
    let item_code: String = sqlx::query_scalar("SELECT item_code FROM event_shop_items WHERE id=$1").bind(body.shop_item_id).fetch_one(&mut *tx).await?;
    sqlx::query("INSERT INTO inventory_items (user_id, template_id) SELECT $1, id FROM item_templates WHERE code=$2 LIMIT 1").bind(user.user_id).bind(&item_code).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"bought":item_code,"cost":cost})))
}
