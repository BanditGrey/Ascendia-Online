use std::sync::Arc;

use actix_web::{web, HttpResponse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{auth::middleware::AuthenticatedUser, error::AppResult, player::progression::grant_leader_experience, ranking::refresh_user, state::AppState};

const STANDARD_CAP_SECONDS: i64 = 12 * 60 * 60;
const VIP_CAP_SECONDS: i64 = 24 * 60 * 60;

#[derive(Deserialize)]
struct ClaimRequest { idempotency_key: Uuid }
#[derive(FromRow)]
struct ClaimRow { elapsed_seconds: i32, gold_reward: i64, experience_reward: i64 }
#[derive(Serialize)]
struct ClaimResponse { idempotency_key: Uuid, elapsed_seconds: i32, gold: i64, experience: i64, level_up: Option<i16>, replayed: bool }

pub fn configure(cfg: &mut web::ServiceConfig) { cfg.route("/offline-rewards/claim", web::post().to(claim)); }

/// Calcula 50% da produção pela última fase concluída. O recibo torna retries seguros.
async fn claim(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, body: web::Json<ClaimRequest>) -> AppResult<HttpResponse> {
    let mut tx = state.db.begin().await?;
    // Serializa claims por jogador; o cliente não define timestamps, fase ou valores.
    sqlx::query("INSERT INTO offline_reward_state (user_id) VALUES ($1) ON CONFLICT (user_id) DO NOTHING").bind(user.user_id).execute(&mut *tx).await?;
    let (last_claim_at, vip_level): (DateTime<Utc>, i16) = sqlx::query_as("SELECT s.last_claim_at,u.vip_level FROM offline_reward_state s JOIN users u ON u.id=s.user_id WHERE s.user_id=$1 FOR UPDATE OF s")
        .bind(user.user_id).fetch_one(&mut *tx).await?;
    // A verificação ocorre depois do lock: duas tentativas com a mesma chave retornam o mesmo recibo.
    let existing: Option<ClaimRow> = sqlx::query_as("SELECT elapsed_seconds,gold_reward,experience_reward FROM offline_reward_claims WHERE user_id=$1 AND idempotency_key=$2")
        .bind(user.user_id).bind(body.idempotency_key).fetch_optional(&mut *tx).await?;
    if let Some(claim) = existing {
        tx.commit().await?;
        return Ok(HttpResponse::Ok().json(ClaimResponse { idempotency_key: body.idempotency_key, elapsed_seconds: claim.elapsed_seconds, gold: claim.gold_reward, experience: claim.experience_reward, level_up: None, replayed: true }));
    }
    let cap = if vip_level > 0 { VIP_CAP_SECONDS } else { STANDARD_CAP_SECONDS };
    let elapsed = (Utc::now() - last_claim_at).num_seconds().clamp(0, cap);
    let stage: i16 = sqlx::query_scalar("SELECT max_stage FROM stage_progress WHERE user_id=$1").bind(user.user_id).fetch_optional(&mut *tx).await?.unwrap_or(0);
    let (gold, experience) = rewards(stage, elapsed);
    sqlx::query("UPDATE users SET gold=gold+$2 WHERE id=$1").bind(user.user_id).bind(gold).execute(&mut *tx).await?;
    let level_up = if experience > 0 { grant_leader_experience(&mut tx, user.user_id, experience).await? } else { None };
    sqlx::query("UPDATE offline_reward_state SET last_claim_at=now(),updated_at=now() WHERE user_id=$1").bind(user.user_id).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO offline_reward_claims (id,user_id,idempotency_key,elapsed_seconds,gold_reward,experience_reward) VALUES ($1,$2,$3,$4,$5,$6)")
        .bind(Uuid::new_v4()).bind(user.user_id).bind(body.idempotency_key).bind(elapsed as i32).bind(gold).bind(experience).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO audit_logs (actor_user_id,action,metadata) VALUES ($1,'OFFLINE_REWARDS_CLAIMED',$2)")
        .bind(user.user_id).bind(serde_json::json!({"idempotency_key":body.idempotency_key,"stage":stage,"elapsed_seconds":elapsed,"gold":gold,"experience":experience})).execute(&mut *tx).await?;
    tx.commit().await?;
    if experience > 0 { refresh_user(&state, user.user_id).await; }
    Ok(HttpResponse::Ok().json(ClaimResponse { idempotency_key: body.idempotency_key, elapsed_seconds: elapsed as i32, gold, experience, level_up, replayed: false }))
}

fn rewards(stage: i16, elapsed_seconds: i64) -> (i64, i64) {
    if stage <= 0 || elapsed_seconds <= 0 { return (0, 0); }
    let hours = elapsed_seconds as f64 / 3_600.0;
    // 50% da produção ativa da fase; o balanceamento nunca vem do cliente.
    let gold = ((10.0 + f64::from(stage) * 2.0) * hours * 0.5).floor() as i64;
    let experience = ((5.0 + f64::from(stage)) * hours * 0.5).floor() as i64;
    (gold, experience)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recompensa_respeita_metade_e_fase() { assert_eq!(rewards(0, 43_200), (0, 0)); assert_eq!(rewards(1, 3_600), (6, 3)); }
    #[test]
    fn teto_normal_e_vip_sao_diferentes() { assert_eq!(STANDARD_CAP_SECONDS, 43_200); assert_eq!(VIP_CAP_SECONDS, 86_400); }
}
