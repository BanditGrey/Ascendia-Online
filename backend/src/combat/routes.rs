use std::sync::Arc;

use actix_web::{web, HttpResponse};
use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{auth::middleware::AuthenticatedUser, combat::{drops::{roll_rarity, Rarity}, engine::{duel, enemy_for_stage, FighterStats}}, error::{AppError, AppResult}, state::AppState};

#[derive(Debug, Deserialize)]
pub struct StartCombat { pub stage: u16, pub difficulty: Difficulty }

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Difficulty { Normal, Hard, Inferno, Chaos }

#[derive(Debug, Serialize)]
pub struct StageResult {
    pub combat_id: Uuid,
    pub stage: u16,
    pub victory: bool,
    pub duration_ms: u64,
    pub gold: i64,
    pub experience: i64,
    pub seed: u64,
    pub drop_rarity: Option<Rarity>,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/combat").route("/start", web::post().to(start)));
}

async fn start(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, body: web::Json<StartCombat>) -> AppResult<HttpResponse> {
    if !(1..=50).contains(&body.stage) {
        return Err(AppError::Validation("o MVP contém as fases 1 a 50".into()));
    }
    let mut tx = state.db.begin().await?;
    let max_stage: i32 = sqlx::query_scalar("SELECT max_stage FROM stage_progress WHERE user_id=$1 FOR UPDATE")
        .bind(user.user_id).fetch_optional(&mut *tx).await?.unwrap_or(0);
    if body.stage as i32 > max_stage + 1 {
        return Err(AppError::Validation("conclua a fase anterior primeiro".into()));
    }

    let stats: Option<(i64, i64, i64, f64, f64, f64, f64, f64, f64, f64)> = sqlx::query_as(
        "SELECT cs.hp,cs.attack,cs.defense,cs.attack_speed,cs.crit_rate,cs.crit_damage,cs.accuracy,cs.dodge,cs.penetration,cs.luck FROM characters c JOIN character_stats cs ON cs.character_id=c.id WHERE c.user_id=$1 AND c.is_leader=true"
    ).bind(user.user_id).fetch_optional(&mut *tx).await?;
    let s = stats.ok_or(AppError::NotFound)?;
    let hero = FighterStats { hp:s.0, attack:s.1, defense:s.2, attack_speed:s.3, crit_rate:s.4, crit_damage:s.5, accuracy:s.6, dodge:s.7, penetration:s.8 };
    let mut enemy = enemy_for_stage(body.stage);
    let difficulty_multiplier = match &body.difficulty { Difficulty::Normal => 1.0, Difficulty::Hard => 1.25, Difficulty::Inferno => 1.6, Difficulty::Chaos => 2.2 };
    enemy.hp = (enemy.hp as f64 * difficulty_multiplier) as i64;
    enemy.attack = (enemy.attack as f64 * difficulty_multiplier) as i64;

    // A seed nasce no servidor; cliente nenhum pode escolher resultados ou drops.
    let seed = rand::thread_rng().gen::<u64>();
    let result = duel(&hero, &enemy, seed, 180_000);
    let combat_id = Uuid::new_v4();
    let (gold, experience) = if result.victory {
        let reward_multiplier = match &body.difficulty { Difficulty::Normal => 1.0, Difficulty::Hard => 1.2, Difficulty::Inferno => 1.5, Difficulty::Chaos => 2.0 };
        (((25 + body.stage as i64 * 4) as f64 * reward_multiplier) as i64, 15 + body.stage as i64 * 3)
    } else { (0, 0) };

    sqlx::query("INSERT INTO combat_runs (id,user_id,stage,difficulty,seed,victory,duration_ms,damage_dealt,damage_taken,gold_reward,experience_reward) VALUES ($1,$2,$3,$4::text::combat_difficulty,$5,$6,$7,$8,$9,$10,$11)")
        .bind(combat_id).bind(user.user_id).bind(body.stage as i32).bind(difficulty_name(&body.difficulty))
        .bind(seed as i64).bind(result.victory).bind(result.duration_ms as i64).bind(result.damage_dealt).bind(result.damage_taken).bind(gold).bind(experience)
        .execute(&mut *tx).await?;
    let difficulty_drop_bonus = match &body.difficulty {
        Difficulty::Normal => 0.0,
        Difficulty::Hard => 0.02,
        Difficulty::Inferno => 0.05,
        Difficulty::Chaos => 0.10,
    };
    let drop_rarity = result.victory.then(|| {
        roll_rarity(seed ^ 0xA5CE_D1A0, s.9, difficulty_drop_bonus).limited_to_stage(body.stage)
    });
    if let Some(rarity) = drop_rarity {
        sqlx::query("INSERT INTO inventory_items (user_id,template_id,trade_locked_until) SELECT $1,id,now()+interval '24 hours' FROM item_templates WHERE code=$2")
            .bind(user.user_id).bind(rarity.code()).execute(&mut *tx).await?;
    }
    if result.victory {
        sqlx::query("INSERT INTO stage_progress (user_id,max_stage,total_stars) VALUES ($1,$2,1) ON CONFLICT (user_id) DO UPDATE SET max_stage=GREATEST(stage_progress.max_stage,EXCLUDED.max_stage), total_stars=stage_progress.total_stars+CASE WHEN EXCLUDED.max_stage>stage_progress.max_stage THEN 1 ELSE 0 END, updated_at=now()")
            .bind(user.user_id).bind(body.stage as i32).execute(&mut *tx).await?;
        sqlx::query("UPDATE users SET gold=gold+$2 WHERE id=$1").bind(user.user_id).bind(gold).execute(&mut *tx).await?;
    }
    sqlx::query("INSERT INTO audit_logs (actor_user_id,action,metadata) VALUES ($1,'COMBAT_RESOLVED',$2)")
        .bind(user.user_id).bind(serde_json::json!({"combat_id":combat_id,"stage":body.stage,"victory":result.victory,"seed":seed})).execute(&mut *tx).await?;
    tx.commit().await?;

    Ok(HttpResponse::Ok().json(StageResult { combat_id, stage: body.stage, victory: result.victory, duration_ms: result.duration_ms, gold, experience, seed, drop_rarity }))
}

fn difficulty_name(value: &Difficulty) -> &'static str {
    match value { Difficulty::Normal=>"normal", Difficulty::Hard=>"hard", Difficulty::Inferno=>"inferno", Difficulty::Chaos=>"chaos" }
}
