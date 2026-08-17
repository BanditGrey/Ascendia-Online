use std::sync::Arc;

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::{auth::middleware::AuthenticatedUser, error::{AppError, AppResult}, inventory::stats::recalculate, state::AppState};

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum CosmeticType { Wings, Mount, Pet, Aura, Mask, Trail, HitEffect, Frame }
impl CosmeticType {
    fn code(&self) -> &'static str {
        match self {
            Self::Wings => "wings", Self::Mount => "mount", Self::Pet => "pet", Self::Aura => "aura",
            Self::Mask => "mask", Self::Trail => "trail", Self::HitEffect => "hit_effect", Self::Frame => "frame",
        }
    }
    fn max_tier(&self) -> i16 { 8 }
    fn display_name(&self) -> &'static str {
        match self {
            Self::Wings => "Asas", Self::Mount => "Montaria", Self::Pet => "Pet", Self::Aura => "Aura",
            Self::Mask => "Máscara", Self::Trail => "Trail", Self::HitEffect => "Hit Effect", Self::Frame => "Frame",
        }
    }
}
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

// Custos fiéis à especificação da prompt: 10→100 por estrela, 550 por tier.
// Essências por transição de tier e fase necessária para desbloquear.
fn fragment_cost(stars: i16) -> i32 { match stars { 0=>10,1=>20,2=>30,3=>40,4=>50,5=>60,6=>70,7=>80,8=>90,9=>100,_=>0 } }
fn essence_cost(current_tier: i16) -> i32 { match current_tier { 1=>1,2=>3,3=>5,4=>10,5=>20,6=>50,7=>100,_=>999 } }
fn phase_required(tier: i16) -> i16 { match tier { 1=>10,2=>50,3=>100,4=>150,5=>200,6=>300,7=>400,8=>500,_=>500 } }

async fn upgrade(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, body: web::Json<UpgradeRequest>) -> AppResult<HttpResponse> {
    let kind = body.cosmetic_type.code();
    let mut tx = state.db.begin().await?;
    // Garante linha existente para o tipo solicitado.
    sqlx::query("INSERT INTO cosmetic_progress (user_id,cosmetic_type) VALUES ($1,$2) ON CONFLICT DO NOTHING").bind(user.user_id).bind(kind).execute(&mut *tx).await?;
    let row: CosmeticView = sqlx::query_as("SELECT cosmetic_type,tier,stars,fragments,essences FROM cosmetic_progress WHERE user_id=$1 AND cosmetic_type=$2 FOR UPDATE").bind(user.user_id).bind(kind).fetch_one(&mut *tx).await?;
    if row.tier >= body.cosmetic_type.max_tier() && row.stars >= 10 { return Err(AppError::Validation(format!("{} já está no máximo T8 ★10", body.cosmetic_type.display_name()))); }

    // Fluxo em duas etapas: estrelas 0→10 com fragmentos; tier up consome essências.
    let (next_tier, next_stars, fragments_spent, essences_spent, tier_up) = if row.stars < 10 {
        let cost = fragment_cost(row.stars);
        if row.fragments < cost { return Err(AppError::Validation(format!("fragmentos insuficientes: são necessários {cost}")) ); }
        (row.tier, row.stars + 1, cost, 0, false)
    } else {
        // stars == 10 exige evolução de tier
        if row.tier >= body.cosmetic_type.max_tier() { return Err(AppError::Validation("tier máximo atingido".into())); }
        let need_essence = essence_cost(row.tier);
        if row.essences < need_essence { return Err(AppError::Validation(format!("essências insuficientes: são necessárias {need_essence}"))); }
        let required_phase = phase_required(row.tier + 1);
        let max_stage: i16 = sqlx::query_scalar("SELECT COALESCE(max_stage,0)::smallint FROM stage_progress WHERE user_id=$1").bind(user.user_id).fetch_optional(&mut *tx).await?.unwrap_or(0);
        if max_stage < required_phase { return Err(AppError::Validation(format!("fase {required_phase} necessária para desbloquear T{}", row.tier + 1))); }
        (row.tier + 1, 0, 0, need_essence, true)
    };

    if tier_up {
        sqlx::query("UPDATE cosmetic_progress SET tier=$3,stars=$4,essences=essences-$5 WHERE user_id=$1 AND cosmetic_type=$2").bind(user.user_id).bind(kind).bind(next_tier).bind(next_stars).bind(essences_spent).execute(&mut *tx).await?;
    } else {
        sqlx::query("UPDATE cosmetic_progress SET stars=$3,fragments=fragments-$4 WHERE user_id=$1 AND cosmetic_type=$2").bind(user.user_id).bind(kind).bind(next_stars).bind(fragments_spent).execute(&mut *tx).await?;
    }
    // Bônus globais do Líder afetam todos os personagens: recalcula cada um.
    let characters: Vec<uuid::Uuid> = sqlx::query_scalar("SELECT id FROM characters WHERE user_id=$1").bind(user.user_id).fetch_all(&mut *tx).await?;
    for character_id in characters { recalculate(&mut tx, user.user_id, character_id).await?; }
    sqlx::query("INSERT INTO audit_logs (actor_user_id,action,metadata) VALUES ($1,'COSMETIC_UPGRADED',$2)").bind(user.user_id).bind(serde_json::json!({"type":kind,"tier":next_tier,"stars":next_stars,"tier_up":tier_up,"fragments_spent":fragments_spent,"essences_spent":essences_spent})).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(UpgradeResult { cosmetic_type: kind.into(), tier: next_tier, stars: next_stars, fragments_spent, tier_up }))
}
