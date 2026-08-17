use std::sync::Arc;
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{auth::middleware::AuthenticatedUser, error::{AppError, AppResult}, inventory::stats::recalculate, state::AppState};

#[derive(FromRow, Serialize)]
struct SkillTreeRow { class: String, subclass: String, branch: String, skill_code: String, max_level: i16 }

#[derive(FromRow, Serialize)]
struct CharacterSkill { skill_code: String, level: i16, branch: String, max_level: i16 }

#[derive(Deserialize)]
struct AllocateRequest { skill_code: String }

#[derive(Serialize)]
struct SkillPointsView { character_id: Uuid, available: i16, total_earned: i16, level: i16 }

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/characters/{character_id}/skills")
        .route("", web::get().to(list))
        .route("/allocate", web::post().to(allocate))
        .route("/reset", web::post().to(reset))
    )
    .service(web::scope("/skills")
        .route("/tree", web::get().to(tree))
    );
}

async fn tree(state: web::Data<Arc<AppState>>, _user: AuthenticatedUser) -> AppResult<HttpResponse> {
    let rows: Vec<SkillTreeRow> = sqlx::query_as("SELECT class::text as class, subclass, branch, skill_code, max_level FROM skill_trees ORDER BY class, subclass, branch")
        .fetch_all(&state.db).await?;
    Ok(HttpResponse::Ok().json(rows))
}

async fn list(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, character_id: web::Path<Uuid>) -> AppResult<HttpResponse> {
    let cid = character_id.into_inner();
    ensure_owner(&state, user.user_id, cid).await?;
    let points: Option<SkillPointsView> = sqlx::query_as("SELECT character_id, available, total_earned, (SELECT level FROM characters WHERE id=$1) as level FROM character_skill_points WHERE character_id=$1")
        .bind(cid).fetch_optional(&state.db).await?;
    // Auto-seed points based on level if not exists: 1 per level
    let points = if let Some(p) = points { p } else {
        let level: i16 = sqlx::query_scalar("SELECT level FROM characters WHERE id=$1 AND user_id=$2").bind(cid).bind(user.user_id).fetch_one(&state.db).await?;
        let available = (level - 1).max(0);
        sqlx::query("INSERT INTO character_skill_points (character_id, available, total_earned) VALUES ($1,$2,$2) ON CONFLICT (character_id) DO NOTHING").bind(cid).bind(available).execute(&state.db).await?;
        SkillPointsView { character_id: cid, available, total_earned: available, level }
    };
    let skills: Vec<CharacterSkill> = sqlx::query_as("SELECT cs.skill_code, cs.level, st.branch, st.max_level FROM character_skills cs JOIN skill_trees st ON st.skill_code=cs.skill_code WHERE cs.character_id=$1")
        .bind(cid).fetch_all(&state.db).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"points": points, "skills": skills})))
}

async fn allocate(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, character_id: web::Path<Uuid>, body: web::Json<AllocateRequest>) -> AppResult<HttpResponse> {
    let cid = character_id.into_inner();
    ensure_owner(&state, user.user_id, cid).await?;
    let mut tx = state.db.begin().await?;
    // Verifica árvore
    let tree: Option<(String, String, i16)> = sqlx::query_as("SELECT branch, subclass, max_level FROM skill_trees WHERE skill_code=$1")
        .bind(&body.skill_code).fetch_optional(&mut *tx).await?;
    let (branch, subclass, max_level) = tree.ok_or(AppError::Validation("skill_code inválido".into()))?;
    // Verifica classe/subclasse do personagem
    let (class, char_sub): (String, String) = sqlx::query_as("SELECT class::text, subclass FROM characters WHERE id=$1 FOR UPDATE").bind(cid).fetch_one(&mut *tx).await?;
    // Simples validação: subclass deve bater
    if char_sub != subclass {
        // Para comadante, permitir qualquer? Mas vamos ser estritos para demo: permitir se class igual
        let allowed: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM skill_trees WHERE class=$1::text::character_class AND subclass=$2 AND skill_code=$3)")
            .bind(&class).bind(&char_sub).bind(&body.skill_code).fetch_one(&mut *tx).await?;
        if !allowed { return Err(AppError::Validation("skill não pertence à subclasse do personagem".into())); }
    }
    let points: Option<(i16, i16)> = sqlx::query_as("SELECT available, total_earned FROM character_skill_points WHERE character_id=$1 FOR UPDATE").bind(cid).fetch_optional(&mut *tx).await?;
    let (available, _) = points.unwrap_or((0,0));
    if available <= 0 { return Err(AppError::Validation("sem pontos de skill disponíveis (ganha 1 por level)".into())); }
    let cur: i16 = sqlx::query_scalar("SELECT COALESCE(level,0) FROM character_skills WHERE character_id=$1 AND skill_code=$2").bind(cid).bind(&body.skill_code).fetch_optional(&mut *tx).await?.unwrap_or(0);
    if cur >= max_level { return Err(AppError::Validation("skill já no nível máximo".into())); }
    sqlx::query("INSERT INTO character_skills (character_id, skill_code, level) VALUES ($1,$2,1) ON CONFLICT (character_id, skill_code) DO UPDATE SET level=character_skills.level+1")
        .bind(cid).bind(&body.skill_code).execute(&mut *tx).await?;
    sqlx::query("UPDATE character_skill_points SET available=available-1, updated_at=now() WHERE character_id=$1").bind(cid).execute(&mut *tx).await?;
    // Recalcula stats se skill dá bônus (aplicado via recalculate que lê skills)
    recalculate(&mut tx, user.user_id, cid).await?;
    sqlx::query("INSERT INTO audit_logs (actor_user_id,action,metadata) VALUES ($1,'SKILL_ALLOCATED',$2)").bind(user.user_id).bind(serde_json::json!({"character_id":cid,"skill_code":body.skill_code,"branch":branch})).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"allocated":body.skill_code,"level":cur+1})))
}

async fn reset(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, character_id: web::Path<Uuid>) -> AppResult<HttpResponse> {
    let cid = character_id.into_inner();
    ensure_owner(&state, user.user_id, cid).await?;
    let mut tx = state.db.begin().await?;
    let level: i16 = sqlx::query_scalar("SELECT level FROM characters WHERE id=$1").bind(cid).fetch_one(&mut *tx).await?;
    let total = (level - 1).max(0);
    sqlx::query("DELETE FROM character_skills WHERE character_id=$1").bind(cid).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO character_skill_points (character_id, available, total_earned) VALUES ($1,$2,$2) ON CONFLICT (character_id) DO UPDATE SET available=$2, total_earned=$2, updated_at=now()")
        .bind(cid).bind(total).execute(&mut *tx).await?;
    recalculate(&mut tx, user.user_id, cid).await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"reset":true,"available":total})))
}

async fn ensure_owner(state: &AppState, user_id: Uuid, character_id: Uuid) -> AppResult<()> {
    let ok: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM characters WHERE id=$1 AND user_id=$2)").bind(character_id).bind(user_id).fetch_one(&state.db).await?;
    if ok { Ok(()) } else { Err(AppError::NotFound) }
}
