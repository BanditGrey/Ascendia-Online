use std::sync::Arc;
use actix_web::{web, HttpResponse};
use rand::{Rng, seq::SliceRandom};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{auth::middleware::AuthenticatedUser, error::{AppError, AppResult}, inventory::stats::recalculate, state::AppState};

#[derive(Deserialize)]
struct EnchantRequest { inventory_item_id: Uuid, locked_stats: Option<Vec<String>> }

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/enchant")
        .route("", web::post().to(enchant))
    );
}

async fn enchant(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, body: web::Json<EnchantRequest>) -> AppResult<HttpResponse> {
    let mut tx = state.db.begin().await?;
    let owner: Option<Uuid> = sqlx::query_scalar("SELECT user_id FROM inventory_items WHERE id=$1 FOR UPDATE").bind(body.inventory_item_id).fetch_optional(&mut *tx).await?;
    if owner.ok_or(AppError::NotFound)? != user.user_id { return Err(AppError::NotFound); }
    // Custo: 1 Scroll + 200 Gold
    let scrolls: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(quantity),0)::bigint FROM enchant_scrolls WHERE user_id=$1").bind(user.user_id).fetch_one(&mut *tx).await?;
    if scrolls < 1 { return Err(AppError::Validation("sem Scroll de Enchant (drop/evento)".into())); }
    let gold: i64 = sqlx::query_scalar("SELECT gold FROM users WHERE id=$1 FOR UPDATE").bind(user.user_id).fetch_one(&mut *tx).await?;
    if gold < 200 { return Err(AppError::Validation("200 Gold necessários".into())); }
    // Gera novos stats secundários aleatórios, respeitando locks
    let current: serde_json::Value = sqlx::query_scalar("SELECT rolled_stats FROM inventory_items WHERE id=$1").bind(body.inventory_item_id).fetch_one(&mut *tx).await?;
    let locked = body.locked_stats.clone().unwrap_or_default();
    let mut new_stats = current.clone();
    let mut rng = rand::thread_rng();
    let pool = ["crit_rate","crit_damage","attack_speed","luck","accuracy","dodge","penetration","hp","defense"];
    // Reroll 2-3 stats não travados
    let to_reroll = 2 + rng.gen_range(0..2);
    let mut candidates: Vec<&str> = pool.iter().filter(|k| !locked.contains(&k.to_string())).copied().collect();
    candidates.shuffle(&mut rng);
    for key in candidates.into_iter().take(to_reroll as usize) {
        let val = match key {
            "crit_rate" => rng.gen_range(0.01..0.04),
            "crit_damage" => 1.5 + rng.gen_range(0.1..0.4),
            "attack_speed" => rng.gen_range(0.05..0.15),
            "luck" => rng.gen_range(0.01..0.03),
            "accuracy" => rng.gen_range(0.02..0.05),
            "dodge" => rng.gen_range(0.02..0.05),
            "penetration" => rng.gen_range(0.02..0.06),
            "hp" => rng.gen_range(20.0..80.0),
            "defense" => rng.gen_range(5.0..20.0),
            _=> 0.0,
        };
        if let Some(obj) = new_stats.as_object_mut() {
            obj.insert(key.to_string(), serde_json::json!(val));
        }
    }
    sqlx::query("UPDATE inventory_items SET rolled_stats=$2 WHERE id=$1").bind(body.inventory_item_id).bind(&new_stats).execute(&mut *tx).await?;
    sqlx::query("UPDATE enchant_scrolls SET quantity=quantity-1 WHERE user_id=$1 AND quantity>0").bind(user.user_id).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM enchant_scrolls WHERE user_id=$1 AND quantity<=0").bind(user.user_id).execute(&mut *tx).await?;
    sqlx::query("UPDATE users SET gold=gold-200 WHERE id=$1").bind(user.user_id).execute(&mut *tx).await?;
    // Recalcula se equipado
    if let Some(char_id) = sqlx::query_scalar::<_,Uuid>("SELECT character_id FROM equipment_slots WHERE inventory_item_id=$1").bind(body.inventory_item_id).fetch_optional(&mut *tx).await? {
        recalculate(&mut tx, user.user_id, char_id).await?;
    }
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"enchanted":body.inventory_item_id,"rolled_stats":new_stats})))
}
