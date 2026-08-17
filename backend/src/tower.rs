use std::sync::Arc;
use actix_web::{web, HttpResponse};
use rand::Rng;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{auth::middleware::AuthenticatedUser, combat::{drops::{roll_rarity, Rarity}, engine::FighterStats, waves::{resolve_stage, SquadMember}}, error::{AppError, AppResult}, player::progression::grant_leader_experience, state::AppState, squad::apply_formation_and_synergy};

#[derive(Deserialize)]
struct ChallengeRequest { }

#[derive(Deserialize)]
struct TowerRankingQuery { #[serde(default)] offset: usize, #[serde(default="default_limit")] limit: usize }
fn default_limit()->usize{20}
#[derive(Serialize)]
struct TowerStatus { current_floor: i32, best_floor: i32, next_floor: i32, is_boss: bool, rewards_preview: String }

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/tower")
        .route("/status", web::get().to(status))
        .route("/challenge", web::post().to(challenge))
        .route("/ranking", web::get().to(ranking))
    );
}

async fn status(state: web::Data<Arc<AppState>>, user: AuthenticatedUser) -> AppResult<HttpResponse> {
    let row: Option<(i32,i32)> = sqlx::query_as("SELECT current_floor, best_floor FROM tower_progress WHERE user_id=$1").bind(user.user_id).fetch_optional(&state.db).await?;
    let (cur,best)= row.unwrap_or((0,0));
    let next=cur+1;
    Ok(HttpResponse::Ok().json(TowerStatus { current_floor:cur, best_floor:best, next_floor:next, is_boss: next%10==0, rewards_preview: if next%10==0 { "Boss: Essência + Box Épico" } else { "Fragmentos + Gold" }.into() }))
}

async fn challenge(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, _body: web::Json<ChallengeRequest>) -> AppResult<HttpResponse> {
    let mut tx = state.db.begin().await?;
    sqlx::query("INSERT INTO tower_progress (user_id) VALUES ($1) ON CONFLICT (user_id) DO NOTHING").bind(user.user_id).execute(&mut *tx).await?;
    let (cur,best): (i32,i32) = sqlx::query_as("SELECT current_floor, best_floor FROM tower_progress WHERE user_id=$1 FOR UPDATE").bind(user.user_id).fetch_one(&mut *tx).await?;
    let next= (cur+1) as u16;
    // Usa engine de combate com scaling infinito: tower floor como stage + bônus
    let rows: Vec<(Uuid,i16,String,String,i64,i64,i64,f64,f64,f64,f64,f64,f64,f64)> = sqlx::query_as("SELECT c.id, ss.slot, c.class::text, s.formation, cs.hp, cs.attack, cs.defense, cs.attack_speed, cs.crit_rate, cs.crit_damage, cs.accuracy, cs.dodge, cs.penetration, cs.luck FROM squads s JOIN squad_slots ss ON ss.squad_id=s.id JOIN characters c ON c.id=ss.character_id JOIN character_stats cs ON cs.character_id=c.id WHERE s.user_id=$1 AND s.is_active=true ORDER BY ss.slot FOR UPDATE OF s")
        .bind(user.user_id).fetch_all(&mut *tx).await?;
    if rows.is_empty() { return Err(AppError::Validation("squad vazio".into())); }
    let formation = rows[0].3.clone();
    let luck = rows.iter().map(|r| r.13).sum::<f64>()/rows.len() as f64;
    let mut squad: Vec<SquadMember> = rows.into_iter().map(|r| SquadMember{ character_id:r.0.to_string(), slot:r.1, class:r.2, stats:FighterStats{ hp:r.4, attack:r.5, defense:r.6, attack_speed:r.7, crit_rate:r.8, crit_damage:r.9, accuracy:r.10, dodge:r.11, penetration:r.12 }}).collect();
    apply_formation_and_synergy(&mut squad, &formation);
    let seed = rand::thread_rng().gen::<u64>();
    // Torre usa dificuldade 1.0 + floor*0.02 escalonado
    let difficulty = 1.0 + (next as f64)*0.018;
    let result = resolve_stage(&squad, next.min(500), difficulty, seed);
    let victory = result.victory;
    let new_cur = if victory { cur+1 } else { cur };
    let new_best = new_cur.max(best);
    sqlx::query("UPDATE tower_progress SET current_floor=$2, best_floor=$3, updated_at=now() WHERE user_id=$1").bind(user.user_id).bind(new_cur).bind(new_best).execute(&mut *tx).await?;
    // Rewards
    let (gold, xp) = if victory { (50 + next*5, 20 + next*2) } else { (0,0) };
    if victory {
        sqlx::query("UPDATE users SET gold=gold+$2 WHERE id=$1").bind(user.user_id).bind(gold as i64).execute(&mut *tx).await?;
        grant_leader_experience(&mut tx, user.user_id, xp as i64).await?;
        if next%10==0 {
            for kind in ["wings","mount","pet","aura"] {
                sqlx::query("INSERT INTO cosmetic_progress (user_id,cosmetic_type,essences) VALUES ($1,$2,1) ON CONFLICT (user_id,cosmetic_type) DO UPDATE SET essences=cosmetic_progress.essences+1").bind(user.user_id).bind(kind).execute(&mut *tx).await?;
            }
        }
        // Drop em torre
        let rarity = roll_rarity(seed ^ 0xC0FFEE, luck, 0.05);
        // Não persistir item torre aqui para simplificar, apenas log
        let _ = rarity;
    }
    sqlx::query("INSERT INTO audit_logs (actor_user_id,action,metadata) VALUES ($1,'TOWER_CHALLENGED',$2)").bind(user.user_id).bind(serde_json::json!({"floor":next,"victory":victory,"seed":seed})).execute(&mut *tx).await?;
    tx.commit().await?;
    // Atualiza ranking Redis (tower) via ZSET
    if victory {
        if let Ok(mut conn) = state.redis.get_multiplexed_async_connection().await {
            let _: redis::RedisResult<()> = redis::cmd("ZADD").arg("ranking:tower:v1").arg(new_best).arg(user.user_id.to_string()).query_async(&mut conn).await;
        }
    }
    Ok(HttpResponse::Ok().json(serde_json::json!({"floor":next,"victory":victory,"current_floor":new_cur,"best_floor":new_best,"gold":gold,"xp":xp,"events":result.events})))
}

async fn ranking(state: web::Data<Arc<AppState>>, _user: AuthenticatedUser, q: web::Query<TowerRankingQuery>) -> AppResult<HttpResponse> {
    // Reusa lógica de ranking power mas para torre
    let limit = q.limit.clamp(1, 50);
    let offset = q.offset;
    let mut conn = state.redis.get_multiplexed_async_connection().await?;
    let ids: Vec<String> = redis::cmd("ZREVRANGE").arg("ranking:tower:v1").arg(offset).arg(offset+limit-1).query_async(&mut conn).await?;
    if ids.is_empty() {
        // Rebuild simples: top 50 do PG
        let rows: Vec<(Uuid,String,i32)> = sqlx::query_as("SELECT user_id, (SELECT display_name FROM users WHERE id=user_id), best_floor FROM tower_progress ORDER BY best_floor DESC LIMIT 50")
            .fetch_all(&state.db).await?;
        for (uid,_,floor) in &rows {
            let _: redis::RedisResult<()> = redis::cmd("ZADD").arg("ranking:tower:v1").arg(*floor).arg(uid.to_string()).query_async(&mut conn).await;
        }
    }
    let ids: Vec<String> = redis::cmd("ZREVRANGE").arg("ranking:tower:v1").arg(offset).arg(offset+limit-1).query_async(&mut conn).await?;
    let mut entries = Vec::new();
    for (i, id) in ids.iter().enumerate() {
        if let Ok(uid) = Uuid::parse_str(id) {
            if let Ok(Some((name,floor))) = sqlx::query_as::<_,(String,i32)>("SELECT display_name, (SELECT best_floor FROM tower_progress WHERE user_id=$1) FROM users WHERE id=$1").bind(uid).fetch_optional(&state.db).await {
                entries.push(serde_json::json!({"rank":offset+i+1,"user_id":uid,"display_name":name,"floor":floor}));
            }
        }
    }
    Ok(HttpResponse::Ok().json(serde_json::json!({"entries":entries,"offset":offset,"limit":limit})))
}
