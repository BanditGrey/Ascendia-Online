use std::sync::Arc;
use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::{auth::middleware::AuthenticatedUser, error::{AppError, AppResult}, inventory::stats::recalculate, state::AppState};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/characters/{character_id}/awaken").route("", web::post().to(awaken)));
}

async fn awaken(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, character_id: web::Path<Uuid>) -> AppResult<HttpResponse> {
    let cid = character_id.into_inner();
    let mut tx = state.db.begin().await?;
    let row: Option<(i16,i16,String)> = sqlx::query_as("SELECT level, awakening, class::text FROM characters WHERE id=$1 AND user_id=$2 FOR UPDATE").bind(cid).bind(user.user_id).fetch_optional(&mut *tx).await?;
    let (level, awakening, class) = row.ok_or(AppError::NotFound)?;
    if awakening >= 5 { return Err(AppError::Validation("awakening máximo 5 já atingido".into())); }
    if level < 100 { return Err(AppError::Validation("requer Level 100 para despertar".into())); }
    // Custo: gold escalonado + essências
    let cost_gold = 5000 + awakening as i64 * 5000;
    let gold: i64 = sqlx::query_scalar("SELECT gold FROM users WHERE id=$1 FOR UPDATE").bind(user.user_id).fetch_one(&mut *tx).await?;
    if gold < cost_gold { return Err(AppError::Validation(format!("gold insuficiente: precisa {cost_gold}"))); }
    // Reset para level 1, mantém itens/cosméticos, ganha awakening
    sqlx::query("UPDATE users SET gold=gold-$2 WHERE id=$1").bind(user.user_id).bind(cost_gold).execute(&mut *tx).await?;
    sqlx::query("UPDATE characters SET level=1, experience=0, awakening=awakening+1 WHERE id=$1").bind(cid).execute(&mut *tx).await?;
    // Recalcula base stats para level 1 + awakening multiplier
    let (hp, atk, def) = crate::player::progression::base_for_class(&class, 1);
    sqlx::query("UPDATE character_base_stats SET hp=$2, attack=$3, defense=$4 WHERE character_id=$1").bind(cid).bind(hp).bind(atk).bind(def).execute(&mut *tx).await?;
    // Skill points reset: devolve pontos baseados no novo level (0)
    sqlx::query("DELETE FROM character_skills WHERE character_id=$1").bind(cid).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO character_skill_points (character_id, available, total_earned) VALUES ($1,0,0) ON CONFLICT (character_id) DO UPDATE SET available=0, total_earned=0").bind(cid).execute(&mut *tx).await?;
    recalculate(&mut tx, user.user_id, cid).await?;
    sqlx::query("INSERT INTO awakening_logs (user_id, character_id, from_level, to_awakening) VALUES ($1,$2,$3,$4)").bind(user.user_id).bind(cid).bind(level).bind(awakening+1).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO audit_logs (actor_user_id,action,metadata) VALUES ($1,'AWAKENING',$2)").bind(user.user_id).bind(serde_json::json!({"character_id":cid,"awakening":awakening+1})).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"awakening":awakening+1,"level":1,"cost_gold":cost_gold})))
}
