use std::sync::Arc;

use actix_web::{web, HttpResponse};
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    auth::middleware::AuthenticatedUser,
    error::{AppError, AppResult},
    inventory::stats::{recalculate, CalculatedStats},
    state::AppState,
};

#[derive(Debug, FromRow, Serialize)]
pub struct InventoryItem {
    pub id: Uuid,
    pub template_code: String,
    pub name: String,
    pub slot: Option<String>,
    pub rarity: String,
    pub tier: i16,
    pub quantity: i32,
    pub enhancement: i16,
    pub base_stats: Value,
    pub rolled_stats: Value,
    pub bound: bool,
    pub equipped_by: Option<Uuid>,
    pub equipped_slot: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EquipRequest {
    pub character_id: Uuid,
    pub item_id: Uuid,
    #[serde(default = "default_slot_index")]
    pub slot_index: i16,
}

#[derive(Debug, Deserialize)]
pub struct UnequipRequest {
    pub character_id: Uuid,
    pub slot: String,
    #[serde(default = "default_slot_index")]
    pub slot_index: i16,
}

#[derive(Debug, Deserialize)]
pub struct EnhanceRequest {
    pub item_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct EquipmentResult {
    pub character_id: Uuid,
    pub stats: CalculatedStats,
}

#[derive(Debug, Serialize)]
pub struct EnhanceResult {
    pub item_id: Uuid,
    pub success: bool,
    pub enhancement: i16,
    pub fragments_spent: i64,
}

fn default_slot_index() -> i16 { 1 }

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/inventory")
            .route("", web::get().to(list))
            .route("/equip", web::post().to(equip))
            .route("/unequip", web::post().to(unequip))
            .route("/enhance", web::post().to(enhance)),
    )
    .route("/characters/{character_id}/stats", web::get().to(get_stats));
}

async fn list(state: web::Data<Arc<AppState>>, user: AuthenticatedUser) -> AppResult<HttpResponse> {
    let items = sqlx::query_as::<_, InventoryItem>(
        "SELECT i.id,t.code AS template_code,t.name,t.slot::text AS slot,t.rarity::text AS rarity,t.tier,i.quantity,i.enhancement,t.base_stats,i.rolled_stats,i.bound,e.character_id AS equipped_by,e.slot::text AS equipped_slot FROM inventory_items i JOIN item_templates t ON t.id=i.template_id LEFT JOIN equipment_slots e ON e.inventory_item_id=i.id WHERE i.user_id=$1 ORDER BY i.acquired_at DESC",
    )
    .bind(user.user_id)
    .fetch_all(&state.db)
    .await?;
    Ok(HttpResponse::Ok().json(items))
}

async fn equip(
    state: web::Data<Arc<AppState>>,
    user: AuthenticatedUser,
    body: web::Json<EquipRequest>,
) -> AppResult<HttpResponse> {
    let mut tx = state.db.begin().await?;
    ensure_character_owner(&mut tx, user.user_id, body.character_id).await?;
    let item_slot: Option<Option<String>> = sqlx::query_scalar(
        "SELECT t.slot::text FROM inventory_items i JOIN item_templates t ON t.id=i.template_id WHERE i.id=$1 AND i.user_id=$2 FOR UPDATE OF i",
    )
    .bind(body.item_id)
    .bind(user.user_id)
    .fetch_optional(&mut *tx)
    .await?;
    let slot = item_slot.flatten().ok_or_else(|| AppError::Validation("este item não é equipável".into()))?;
    validate_slot_index(&slot, body.slot_index)?;

    // Permite mover o item atomicamente e devolve ao inventário o item substituído.
    sqlx::query("DELETE FROM equipment_slots WHERE inventory_item_id=$1")
        .bind(body.item_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO equipment_slots (character_id,slot,slot_index,inventory_item_id) VALUES ($1,$2::text::item_slot,$3,$4) ON CONFLICT (character_id,slot,slot_index) DO UPDATE SET inventory_item_id=EXCLUDED.inventory_item_id")
        .bind(body.character_id)
        .bind(&slot)
        .bind(body.slot_index)
        .bind(body.item_id)
        .execute(&mut *tx)
        .await?;
    let stats = recalculate(&mut tx, user.user_id, body.character_id).await?;
    audit(&mut tx, user.user_id, "ITEM_EQUIPPED", serde_json::json!({"item_id":body.item_id,"character_id":body.character_id,"slot":slot,"slot_index":body.slot_index})).await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(EquipmentResult { character_id: body.character_id, stats }))
}

async fn unequip(
    state: web::Data<Arc<AppState>>,
    user: AuthenticatedUser,
    body: web::Json<UnequipRequest>,
) -> AppResult<HttpResponse> {
    validate_slot_index(&body.slot, body.slot_index)?;
    let mut tx = state.db.begin().await?;
    ensure_character_owner(&mut tx, user.user_id, body.character_id).await?;
    let affected = sqlx::query("DELETE FROM equipment_slots WHERE character_id=$1 AND slot=$2::text::item_slot AND slot_index=$3")
        .bind(body.character_id)
        .bind(&body.slot)
        .bind(body.slot_index)
        .execute(&mut *tx)
        .await?
        .rows_affected();
    if affected == 0 { return Err(AppError::NotFound); }
    let stats = recalculate(&mut tx, user.user_id, body.character_id).await?;
    audit(&mut tx, user.user_id, "ITEM_UNEQUIPPED", serde_json::json!({"character_id":body.character_id,"slot":body.slot,"slot_index":body.slot_index})).await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(EquipmentResult { character_id: body.character_id, stats }))
}

async fn enhance(
    state: web::Data<Arc<AppState>>,
    user: AuthenticatedUser,
    body: web::Json<EnhanceRequest>,
) -> AppResult<HttpResponse> {
    let mut tx = state.db.begin().await?;
    let item: Option<(i16, i16)> = sqlx::query_as(
        "SELECT i.enhancement,t.tier FROM inventory_items i JOIN item_templates t ON t.id=i.template_id WHERE i.id=$1 AND i.user_id=$2 FOR UPDATE OF i",
    )
    .bind(body.item_id)
    .bind(user.user_id)
    .fetch_optional(&mut *tx)
    .await?;
    let (current, tier) = item.ok_or(AppError::NotFound)?;
    if current >= 20 { return Err(AppError::Validation("item já está no enhancement máximo".into())); }
    let target = current + 1;
    let cost = i64::from(target) * 5;
    let material = format!("item_fragment_t{tier}");
    let consumed = sqlx::query("UPDATE player_materials SET quantity=quantity-$3,updated_at=now() WHERE user_id=$1 AND material_code=$2 AND quantity >= $3")
        .bind(user.user_id)
        .bind(&material)
        .bind(cost)
        .execute(&mut *tx)
        .await?
        .rows_affected();
    if consumed == 0 { return Err(AppError::Validation(format!("fragmentos insuficientes: são necessários {cost}"))); }
    let chance = enhancement_chance(target);
    let success = rand::thread_rng().gen_bool(chance);
    let resulting_level = if success { target } else { current };
    if success {
        sqlx::query("UPDATE inventory_items SET enhancement=$2 WHERE id=$1")
            .bind(body.item_id)
            .bind(target)
            .execute(&mut *tx)
            .await?;
    }
    let equipped: Option<Uuid> = sqlx::query_scalar("SELECT character_id FROM equipment_slots WHERE inventory_item_id=$1")
        .bind(body.item_id)
        .fetch_optional(&mut *tx)
        .await?;
    if let Some(character_id) = equipped { recalculate(&mut tx, user.user_id, character_id).await?; }
    audit(&mut tx, user.user_id, "ITEM_ENHANCEMENT_ATTEMPT", serde_json::json!({"item_id":body.item_id,"from":current,"to":target,"success":success,"cost":cost})).await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(EnhanceResult { item_id: body.item_id, success, enhancement: resulting_level, fragments_spent: cost }))
}

async fn get_stats(
    state: web::Data<Arc<AppState>>,
    user: AuthenticatedUser,
    character_id: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    let stats: Option<CalculatedStats> = sqlx::query_as::<_, StatsRow>(
        "SELECT cs.hp,cs.attack,cs.defense,cs.attack_speed,cs.crit_rate,cs.crit_damage,cs.luck,cs.accuracy,cs.dodge,cs.penetration,cs.power_rating FROM character_stats cs JOIN characters c ON c.id=cs.character_id WHERE cs.character_id=$1 AND c.user_id=$2",
    )
    .bind(*character_id)
    .bind(user.user_id)
    .fetch_optional(&state.db)
    .await?
    .map(Into::into);
    Ok(HttpResponse::Ok().json(stats.ok_or(AppError::NotFound)?))
}

#[derive(FromRow)]
struct StatsRow { hp:i64, attack:i64, defense:i64, attack_speed:f64, crit_rate:f64, crit_damage:f64, luck:f64, accuracy:f64, dodge:f64, penetration:f64, power_rating:i64 }
impl From<StatsRow> for CalculatedStats {
    fn from(s: StatsRow) -> Self { Self { hp:s.hp, attack:s.attack, defense:s.defense, attack_speed:s.attack_speed, crit_rate:s.crit_rate, crit_damage:s.crit_damage, luck:s.luck, accuracy:s.accuracy, dodge:s.dodge, penetration:s.penetration, power_rating:s.power_rating } }
}

async fn ensure_character_owner(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>, user_id: Uuid, character_id: Uuid) -> AppResult<()> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM characters WHERE id=$1 AND user_id=$2)")
        .bind(character_id).bind(user_id).fetch_one(&mut **tx).await?;
    if exists { Ok(()) } else { Err(AppError::NotFound) }
}

fn validate_slot_index(slot: &str, index: i16) -> AppResult<()> {
    const SLOTS: &[&str] = &["head","main_hand","chest","off_hand","legs","ring","feet","necklace","hands","relic"];
    if !SLOTS.contains(&slot) { return Err(AppError::Validation("slot inválido".into())); }
    let valid = if slot == "ring" { (1..=2).contains(&index) } else { index == 1 };
    if !valid { return Err(AppError::Validation("índice de slot inválido".into())); }
    Ok(())
}

fn enhancement_chance(target: i16) -> f64 {
    match target { 1..=10 => 1.0, 11..=14 => 0.8, 15..=17 => 0.6, 18..=19 => 0.4, 20 => 0.2, _ => 0.0 }
}

async fn audit(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>, actor: Uuid, action: &str, metadata: Value) -> AppResult<()> {
    sqlx::query("INSERT INTO audit_logs (actor_user_id,action,metadata) VALUES ($1,$2,$3)")
        .bind(actor).bind(action).bind(metadata).execute(&mut **tx).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tabela_de_chance_segue_design() {
        assert_eq!(enhancement_chance(10), 1.0);
        assert_eq!(enhancement_chance(11), 0.8);
        assert_eq!(enhancement_chance(15), 0.6);
        assert_eq!(enhancement_chance(18), 0.4);
        assert_eq!(enhancement_chance(20), 0.2);
    }
    #[test]
    fn somente_anel_aceita_segundo_slot() {
        assert!(validate_slot_index("ring", 2).is_ok());
        assert!(validate_slot_index("head", 2).is_err());
    }
}
