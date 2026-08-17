use std::sync::Arc;

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

use crate::{
    auth::middleware::AuthenticatedUser,
    error::{AppError, AppResult},
    inventory::stats::recalculate,
    player::progression::base_for_class,
    state::AppState,
};

#[derive(Debug, FromRow, Serialize)]
struct CharacterView {
    id: Uuid,
    name: String,
    gender: String,
    class: String,
    subclass: String,
    level: i16,
    experience: i64,
    awakening: i16,
    star_rating: i16,
    is_leader: bool,
    power_rating: i64,
}

#[derive(Debug, Deserialize, Validate)]
struct CreateCharacter {
    #[validate(length(min = 3, max = 24))]
    name: String,
    gender: Gender,
    class: SoldierClass,
    subclass: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Gender { Male, Female }
impl Gender { fn as_str(&self) -> &'static str { match self { Self::Male=>"male", Self::Female=>"female" } } }

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum SoldierClass { Warrior, Archer }
impl SoldierClass {
    fn as_str(&self) -> &'static str { match self { Self::Warrior=>"warrior", Self::Archer=>"archer" } }
    fn unlock_level(&self) -> i16 { match self { Self::Warrior=>5, Self::Archer=>15 } }
    fn valid_subclass(&self, value: &str) -> bool {
        match self {
            Self::Warrior => ["guardian","berserker","paladin"].contains(&value),
            Self::Archer => ["marksman","crossbowman","ranger"].contains(&value),
        }
    }
}

#[derive(Debug, Deserialize)]
struct SetSquadSlot { slot: i16, character_id: Option<Uuid> }

#[derive(Debug, Deserialize)]
struct SetFormation { formation: String }

#[derive(Debug, FromRow, Serialize)]
struct SquadSlotView { slot: i16, character_id: Uuid, name: String, class: String, subclass: String, level: i16 }

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/characters")
            .route("", web::get().to(list_characters))
            .route("", web::post().to(create_character)),
    )
    .service(
        web::scope("/squad")
            .route("", web::get().to(get_squad))
            .route("/slot", web::put().to(set_squad_slot))
            .route("/formation", web::put().to(set_formation)),
    );
}

async fn list_characters(state: web::Data<Arc<AppState>>, user: AuthenticatedUser) -> AppResult<HttpResponse> {
    let characters = sqlx::query_as::<_, CharacterView>(
        "SELECT c.id,c.name,c.gender::text AS gender,c.class::text AS class,c.subclass,c.level,c.experience,c.awakening,c.star_rating,c.is_leader,s.power_rating FROM characters c JOIN character_stats s ON s.character_id=c.id WHERE c.user_id=$1 ORDER BY c.is_leader DESC,c.created_at",
    ).bind(user.user_id).fetch_all(&state.db).await?;
    Ok(HttpResponse::Ok().json(characters))
}

async fn create_character(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, body: web::Json<CreateCharacter>) -> AppResult<HttpResponse> {
    body.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    if !body.class.valid_subclass(&body.subclass) { return Err(AppError::Validation("subclasse incompatível com a classe".into())); }
    let mut tx = state.db.begin().await?;
    let leader_level: i16 = sqlx::query_scalar("SELECT level FROM characters WHERE user_id=$1 AND is_leader=true FOR UPDATE")
        .bind(user.user_id).fetch_one(&mut *tx).await?;
    if leader_level < body.class.unlock_level() {
        return Err(AppError::Validation(format!("classe desbloqueia no level {}", body.class.unlock_level())));
    }
    let id = Uuid::new_v4();
    let class = body.class.as_str();
    sqlx::query("INSERT INTO characters (id,user_id,name,gender,class,subclass) VALUES ($1,$2,$3,$4::text::character_gender,$5::text::character_class,$6)")
        .bind(id).bind(user.user_id).bind(body.name.trim()).bind(body.gender.as_str()).bind(class).bind(&body.subclass)
        .execute(&mut *tx).await?;
    let (hp, attack, defense) = base_for_class(class, 1);
    sqlx::query("INSERT INTO character_base_stats (character_id,hp,attack,defense) VALUES ($1,$2,$3,$4)")
        .bind(id).bind(hp).bind(attack).bind(defense).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO character_stats (character_id,hp,attack,defense) VALUES ($1,$2,$3,$4)")
        .bind(id).bind(hp).bind(attack).bind(defense).execute(&mut *tx).await?;
    recalculate(&mut tx, user.user_id, id).await?;
    audit(&mut tx,user.user_id,"CHARACTER_CREATED",serde_json::json!({"character_id":id,"class":class,"subclass":body.subclass})).await?;
    tx.commit().await?;
    Ok(HttpResponse::Created().json(serde_json::json!({"character_id":id})))
}

async fn get_squad(state: web::Data<Arc<AppState>>, user: AuthenticatedUser) -> AppResult<HttpResponse> {
    let slots = sqlx::query_as::<_, SquadSlotView>(
        "SELECT ss.slot,c.id AS character_id,c.name,c.class::text AS class,c.subclass,c.level FROM squads s JOIN squad_slots ss ON ss.squad_id=s.id JOIN characters c ON c.id=ss.character_id WHERE s.user_id=$1 AND s.is_active=true ORDER BY ss.slot",
    ).bind(user.user_id).fetch_all(&state.db).await?;
    Ok(HttpResponse::Ok().json(slots))
}

async fn set_squad_slot(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, body: web::Json<SetSquadSlot>) -> AppResult<HttpResponse> {
    if !(1..=6).contains(&body.slot) { return Err(AppError::Validation("slot deve estar entre 1 e 6".into())); }
    let mut tx = state.db.begin().await?;
    let row: (Uuid, i16) = sqlx::query_as("SELECT s.id,c.level FROM squads s JOIN characters c ON c.user_id=s.user_id AND c.is_leader=true WHERE s.user_id=$1 AND s.is_active=true FOR UPDATE OF s")
        .bind(user.user_id).fetch_one(&mut *tx).await?;
    let required = slot_unlock_level(body.slot);
    if row.1 < required { return Err(AppError::Validation(format!("slot {} desbloqueia no level {required}",body.slot))); }
    match body.character_id {
        None if body.slot == 1 => return Err(AppError::Validation("o Líder não pode ser removido".into())),
        None => { sqlx::query("DELETE FROM squad_slots WHERE squad_id=$1 AND slot=$2").bind(row.0).bind(body.slot).execute(&mut *tx).await?; }
        Some(character_id) => {
            let is_leader: Option<bool> = sqlx::query_scalar("SELECT is_leader FROM characters WHERE id=$1 AND user_id=$2")
                .bind(character_id).bind(user.user_id).fetch_optional(&mut *tx).await?;
            let is_leader = is_leader.ok_or(AppError::NotFound)?;
            if (body.slot == 1) != is_leader { return Err(AppError::Validation("slot 1 é exclusivo do Líder".into())); }
            sqlx::query("DELETE FROM squad_slots WHERE squad_id=$1 AND character_id=$2").bind(row.0).bind(character_id).execute(&mut *tx).await?;
            sqlx::query("INSERT INTO squad_slots (squad_id,slot,character_id) VALUES ($1,$2,$3) ON CONFLICT (squad_id,slot) DO UPDATE SET character_id=EXCLUDED.character_id")
                .bind(row.0).bind(body.slot).bind(character_id).execute(&mut *tx).await?;
        }
    }
    audit(&mut tx,user.user_id,"SQUAD_CHANGED",serde_json::json!({"slot":body.slot,"character_id":body.character_id})).await?;
    tx.commit().await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn set_formation(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, body: web::Json<SetFormation>) -> AppResult<HttpResponse> {
    if !["balanced", "vanguard", "assault"].contains(&body.formation.as_str()) { return Err(AppError::Validation("formação inválida".into())); }
    let mut tx = state.db.begin().await?;
    let updated = sqlx::query("UPDATE squads SET formation=$2 WHERE user_id=$1 AND is_active=true").bind(user.user_id).bind(&body.formation).execute(&mut *tx).await?.rows_affected();
    if updated == 0 { return Err(AppError::NotFound); }
    audit(&mut tx, user.user_id, "SQUAD_FORMATION_CHANGED", serde_json::json!({"formation":body.formation})).await?;
    tx.commit().await?;
    Ok(HttpResponse::NoContent().finish())
}

fn slot_unlock_level(slot: i16) -> i16 { match slot { 1=>1, 2=>5, 3=>15, 4=>35, 5=>55, 6=>80, _=>i16::MAX } }
async fn audit(tx: &mut sqlx::Transaction<'_,sqlx::Postgres>, actor:Uuid, action:&str, metadata:serde_json::Value) -> AppResult<()> {
    sqlx::query("INSERT INTO audit_logs (actor_user_id,action,metadata) VALUES ($1,$2,$3)").bind(actor).bind(action).bind(metadata).execute(&mut **tx).await?; Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn slots_seguem_levels_do_design() { assert_eq!([1,2,3,4,5,6].map(slot_unlock_level),[1,5,15,35,55,80]); }
}
