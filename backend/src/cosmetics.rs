use std::sync::Arc;

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::{auth::middleware::AuthenticatedUser, error::{AppError, AppResult}, inventory::stats::recalculate, state::AppState};

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum CosmeticType { Wings, Mount }
impl CosmeticType { fn code(&self) -> &'static str { match self { Self::Wings => "wings", Self::Mount => "mount" } } fn max_tier(&self) -> i16 { match self { Self::Wings => 3, Self::Mount => 2 } } }
#[derive(FromRow, Serialize)]
struct CosmeticView { cosmetic_type: String, tier: i16, stars: i16, fragments: i32, essences: i32 }
#[derive(Deserialize)]
struct UpgradeRequest { cosmetic_type: CosmeticType }
#[derive(Serialize)]
struct UpgradeResult { cosmetic_type: String, tier: i16, stars: i16, fragments_spent: i32, tier_up: bool }

pub fn configure(cfg: &mut web::ServiceConfig) { cfg.service(web::scope("/cosmetics").route("", web::get().to(list)).route("/upgrade", web::post().to(upgrade))); }

async fn list(state: web::Data<Arc<AppState>>, user: AuthenticatedUser) -> AppResult<HttpResponse> {
    let cosmetics: Vec<CosmeticView> = sqlx::query_as("SELECT cosmetic_type,tier,stars,fragments,essences FROM cosmetic_progress WHERE user_id=$1 ORDER BY cosmetic_type").bind(user.user_id).fetch_all(&state.db).await?;
    Ok(HttpResponse::Ok().json(cosmetics))
}

async fn upgrade(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, body: web::Json<UpgradeRequest>) -> AppResult<HttpResponse> {
    let kind = body.cosmetic_type.code();
    let mut tx = state.db.begin().await?;
    sqlx::query("INSERT INTO cosmetic_progress (user_id,cosmetic_type) VALUES ($1,$2) ON CONFLICT DO NOTHING").bind(user.user_id).bind(kind).execute(&mut *tx).await?;
    let row: CosmeticView = sqlx::query_as("SELECT cosmetic_type,tier,stars,fragments,essences FROM cosmetic_progress WHERE user_id=$1 AND cosmetic_type=$2 FOR UPDATE").bind(user.user_id).bind(kind).fetch_one(&mut *tx).await?;
    if row.tier >= body.cosmetic_type.max_tier() && row.stars >= 10 { return Err(AppError::Validation("cosmético já está no máximo".into())); }
    let cost = i32::from(row.tier) * (row.stars + 1) * 10;
    if row.fragments < cost { return Err(AppError::Validation(format!("fragmentos insuficientes: são necessários {cost}"))); }
    let tier_up = row.stars == 9 && row.tier < body.cosmetic_type.max_tier();
    let stars = if tier_up { 0 } else { row.stars + 1 };
    let tier = if tier_up { row.tier + 1 } else { row.tier };
    sqlx::query("UPDATE cosmetic_progress SET tier=$3,stars=$4,fragments=fragments-$5 WHERE user_id=$1 AND cosmetic_type=$2").bind(user.user_id).bind(kind).bind(tier).bind(stars).bind(cost).execute(&mut *tx).await?;
    let characters: Vec<uuid::Uuid> = sqlx::query_scalar("SELECT id FROM characters WHERE user_id=$1").bind(user.user_id).fetch_all(&mut *tx).await?;
    for character_id in characters { recalculate(&mut tx, user.user_id, character_id).await?; }
    sqlx::query("INSERT INTO audit_logs (actor_user_id,action,metadata) VALUES ($1,'COSMETIC_UPGRADED',$2)").bind(user.user_id).bind(serde_json::json!({"type":kind,"tier":tier,"stars":stars,"cost":cost})).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(UpgradeResult { cosmetic_type: kind.into(), tier, stars, fragments_spent: cost, tier_up }))
}
