use std::sync::Arc;

use actix_web::{web, HttpResponse};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{auth::middleware::AuthenticatedUser, combat::{drops::{roll_rarity, Rarity}, engine::FighterStats, waves::{resolve_stage, SquadMember, WaveEvent}}, error::{AppError, AppResult}, player::progression::grant_leader_experience, ranking::refresh_user, squad::apply_formation_and_synergy, state::AppState};

#[derive(Debug, Deserialize)]
pub struct StartCombat { pub stage: u16, pub difficulty: Difficulty }
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Difficulty { Normal, Hard, Inferno, Chaos }
#[derive(Debug, Serialize)]
pub struct StageResult { pub combat_id: Uuid, pub stage: u16, pub victory: bool, pub duration_ms: u64, pub gold: i64, pub experience: i64, pub seed: u64, pub drop_rarity: Option<Rarity>, pub level_up: Option<i16>, pub stars: i16, pub events: Vec<WaveEvent> }

#[derive(FromRow)]
struct SquadRow { character_id: Uuid, slot: i16, class: String, formation: String, hp: i64, attack: i64, defense: i64, attack_speed: f64, crit_rate: f64, crit_damage: f64, accuracy: f64, dodge: f64, penetration: f64, luck: f64 }

pub fn configure(cfg: &mut web::ServiceConfig) { cfg.service(web::scope("/combat").route("/start", web::post().to(start))); }

async fn start(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, body: web::Json<StartCombat>) -> AppResult<HttpResponse> {
    if !(1..=50).contains(&body.stage) { return Err(AppError::Validation("o MVP contém as fases 1 a 50".into())); }
    let mut tx = state.db.begin().await?;
    let max_stage: i32 = sqlx::query_scalar("SELECT max_stage FROM stage_progress WHERE user_id=$1 FOR UPDATE").bind(user.user_id).fetch_optional(&mut *tx).await?.unwrap_or(0);
    if body.stage as i32 > max_stage + 1 { return Err(AppError::Validation("conclua a fase anterior primeiro".into())); }
    let rows = sqlx::query_as::<_, SquadRow>("SELECT c.id AS character_id,ss.slot,c.class::text AS class,s.formation,cs.hp,cs.attack,cs.defense,cs.attack_speed,cs.crit_rate,cs.crit_damage,cs.accuracy,cs.dodge,cs.penetration,cs.luck FROM squads s JOIN squad_slots ss ON ss.squad_id=s.id JOIN characters c ON c.id=ss.character_id JOIN character_stats cs ON cs.character_id=c.id WHERE s.user_id=$1 AND s.is_active=true ORDER BY ss.slot FOR UPDATE OF s")
        .bind(user.user_id).fetch_all(&mut *tx).await?;
    if rows.is_empty() { return Err(AppError::Validation("o squad ativo não possui integrantes".into())); }
    let luck = rows.iter().map(|row| row.luck).sum::<f64>() / rows.len() as f64;
    let formation = rows[0].formation.clone();
    let mut squad: Vec<SquadMember> = rows.iter().map(|row| SquadMember { character_id: row.character_id.to_string(), slot: row.slot, class: row.class.clone(), stats: FighterStats { hp: row.hp, attack: row.attack, defense: row.defense, attack_speed: row.attack_speed, crit_rate: row.crit_rate, crit_damage: row.crit_damage, accuracy: row.accuracy, dodge: row.dodge, penetration: row.penetration } }).collect();
    apply_formation_and_synergy(&mut squad, &formation);
    let seed = rand::thread_rng().gen::<u64>();
    let result = resolve_stage(&squad, body.stage, difficulty_multiplier(&body.difficulty), seed);
    let combat_id = Uuid::new_v4();
    let stars = stars_for(&result, &squad);
    let (gold, experience) = if result.victory { let multiplier = reward_multiplier(&body.difficulty); (((25 + i64::from(body.stage) * 4) as f64 * multiplier) as i64, 15 + i64::from(body.stage) * 3) } else { (0, 0) };
    let drop_rarity = result.victory.then(|| roll_rarity(seed ^ 0xA5CE_D1A0, luck, drop_bonus(&body.difficulty)).limited_to_stage(body.stage));

    sqlx::query("INSERT INTO combat_sessions (id,user_id,stage,difficulty,seed,squad_snapshot,events) VALUES ($1,$2,$3,$4::text::combat_difficulty,$5,$6,$7)")
        .bind(combat_id).bind(user.user_id).bind(i32::from(body.stage)).bind(difficulty_name(&body.difficulty)).bind(seed as i64).bind(serde_json::json!({"balance_version":"mvp-wave-v1","members":squad})).bind(serde_json::to_value(&result.events).map_err(|e| AppError::Internal(e.to_string()))?).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO combat_runs (id,user_id,stage,difficulty,seed,victory,duration_ms,damage_dealt,damage_taken,gold_reward,experience_reward,combat_session_id) VALUES ($1,$2,$3,$4::text::combat_difficulty,$5,$6,$7,$8,$9,$10,$11,$12)")
        .bind(Uuid::new_v4()).bind(user.user_id).bind(i32::from(body.stage)).bind(difficulty_name(&body.difficulty)).bind(seed as i64).bind(result.victory).bind(result.duration_ms as i64).bind(result.damage_dealt).bind(result.damage_taken).bind(gold).bind(experience).bind(combat_id).execute(&mut *tx).await?;
    if let Some(rarity) = drop_rarity { sqlx::query("INSERT INTO inventory_items (user_id,template_id,trade_locked_until) SELECT $1,id,now()+interval '24 hours' FROM item_templates WHERE code=$2").bind(user.user_id).bind(rarity.code()).execute(&mut *tx).await?; }
    let mut level_up = None;
    if result.victory {
        // Garante a linha antes de projetar o delta de estrelas, inclusive na primeira vitória.
        sqlx::query("INSERT INTO stage_progress (user_id,max_stage,total_stars) VALUES ($1,$2,0) ON CONFLICT (user_id) DO UPDATE SET max_stage=GREATEST(stage_progress.max_stage,EXCLUDED.max_stage),updated_at=now()").bind(user.user_id).bind(i32::from(body.stage)).execute(&mut *tx).await?;
        update_stars(&mut tx, user.user_id, body.stage, difficulty_name(&body.difficulty), stars).await?;
        sqlx::query("UPDATE users SET gold=gold+$2 WHERE id=$1").bind(user.user_id).bind(gold).execute(&mut *tx).await?;
        sqlx::query("INSERT INTO player_materials (user_id,material_code,quantity) VALUES ($1,'item_fragment_t1',1) ON CONFLICT (user_id,material_code) DO UPDATE SET quantity=player_materials.quantity+1,updated_at=now()").bind(user.user_id).execute(&mut *tx).await?;
        level_up = grant_leader_experience(&mut tx, user.user_id, experience).await?;
    }
    sqlx::query("INSERT INTO audit_logs (actor_user_id,action,metadata) VALUES ($1,'COMBAT_RESOLVED',$2)").bind(user.user_id).bind(serde_json::json!({"combat_session_id":combat_id,"stage":body.stage,"victory":result.victory,"seed":seed,"stars":stars})).execute(&mut *tx).await?;
    tx.commit().await?;
    if result.victory { refresh_user(&state, user.user_id).await; }
    Ok(HttpResponse::Ok().json(StageResult { combat_id, stage: body.stage, victory: result.victory, duration_ms: result.duration_ms, gold, experience, seed, drop_rarity, level_up, stars: if result.victory { stars } else { 0 }, events: result.events }))
}

async fn update_stars(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>, user_id: Uuid, stage: u16, difficulty: &str, stars: i16) -> AppResult<()> {
    let previous: i16 = sqlx::query_scalar("SELECT stars FROM stage_stars WHERE user_id=$1 AND stage=$2 AND difficulty=$3::text::combat_difficulty FOR UPDATE").bind(user_id).bind(i32::from(stage)).bind(difficulty).fetch_optional(&mut **tx).await?.unwrap_or(0);
    let best = previous.max(stars);
    sqlx::query("INSERT INTO stage_stars (user_id,stage,difficulty,stars) VALUES ($1,$2,$3::text::combat_difficulty,$4) ON CONFLICT (user_id,stage,difficulty) DO UPDATE SET stars=GREATEST(stage_stars.stars,EXCLUDED.stars),updated_at=now()").bind(user_id).bind(i32::from(stage)).bind(difficulty).bind(best).execute(&mut **tx).await?;
    if best > previous { sqlx::query("UPDATE stage_progress SET total_stars=total_stars+$2 WHERE user_id=$1").bind(user_id).bind(i32::from(best - previous)).execute(&mut **tx).await?; }
    Ok(())
}
fn stars_for(result: &crate::combat::waves::WaveResult, squad: &[SquadMember]) -> i16 { if !result.victory { return 0; } let total_hp: i64 = squad.iter().map(|member| member.stats.hp).sum(); if result.damage_taken <= total_hp / 4 { 3 } else if result.damage_taken <= total_hp / 2 { 2 } else { 1 } }
fn difficulty_name(value: &Difficulty) -> &'static str { match value { Difficulty::Normal=>"normal", Difficulty::Hard=>"hard", Difficulty::Inferno=>"inferno", Difficulty::Chaos=>"chaos" } }
fn difficulty_multiplier(value: &Difficulty) -> f64 { match value { Difficulty::Normal=>1.0, Difficulty::Hard=>1.25, Difficulty::Inferno=>1.6, Difficulty::Chaos=>2.2 } }
fn reward_multiplier(value: &Difficulty) -> f64 { match value { Difficulty::Normal=>1.0, Difficulty::Hard=>1.2, Difficulty::Inferno=>1.5, Difficulty::Chaos=>2.0 } }
fn drop_bonus(value: &Difficulty) -> f64 { match value { Difficulty::Normal=>0.0, Difficulty::Hard=>0.02, Difficulty::Inferno=>0.05, Difficulty::Chaos=>0.10 } }

#[cfg(test)]
mod tests { use super::*; #[test] fn estrelas_respeitam_dano_recebido() { let member = SquadMember { character_id:"a".into(), slot:1, class:"commander".into(), stats:FighterStats { hp:1000,attack:1,defense:1,attack_speed:1.0,crit_rate:0.0,crit_damage:1.5,accuracy:0.0,dodge:0.0,penetration:0.0 } }; let result = crate::combat::waves::WaveResult { victory:true,duration_ms:0,damage_dealt:0,damage_taken:200,events:vec![] }; assert_eq!(stars_for(&result,&[member]),3); } }
