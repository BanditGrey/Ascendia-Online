use std::sync::Arc;
use actix_web::{web, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::{auth::middleware::AuthenticatedUser, error::{AppError, AppResult}, state::AppState};

#[derive(Deserialize)]
struct ChallengeWar { guild_b_id: Uuid }

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/guild-war")
        .route("/challenge", web::post().to(challenge))
        .route("/status", web::get().to(status))
        .route("/territories", web::get().to(territories))
    );
}

async fn challenge(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, body: web::Json<ChallengeWar>) -> AppResult<HttpResponse> {
    let mut tx = state.db.begin().await?;
    let my_guild: Option<Uuid> = sqlx::query_scalar("SELECT guild_id FROM guild_members WHERE user_id=$1").bind(user.user_id).fetch_optional(&mut *tx).await?;
    let my_guild = my_guild.ok_or(AppError::Validation("você não está em guilda".into()))?;
    let role: String = sqlx::query_scalar("SELECT role FROM guild_members WHERE user_id=$1").bind(user.user_id).fetch_one(&mut *tx).await?;
    if !["leader","vice","officer"].contains(&role.as_str()) { return Err(AppError::Validation("apenas Líder/Vice/Oficial pode desafiar".into())); }
    if my_guild == body.guild_b_id { return Err(AppError::Validation("não pode desafiar própria guilda".into())); }
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM guilds WHERE id=$1)").bind(body.guild_b_id).fetch_one(&mut *tx).await?;
    if !exists { return Err(AppError::NotFound); }
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO guild_wars (id, guild_a, guild_b) VALUES ($1,$2,$3)").bind(id).bind(my_guild).bind(body.guild_b_id).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(HttpResponse::Created().json(serde_json::json!({"war_id":id})))
}

async fn status(state: web::Data<Arc<AppState>>, user: AuthenticatedUser) -> AppResult<HttpResponse> {
    let my_guild: Option<Uuid> = sqlx::query_scalar("SELECT guild_id FROM guild_members WHERE user_id=$1").bind(user.user_id).fetch_optional(&state.db).await?;
    if let Some(gid) = my_guild {
        let wars: Vec<(Uuid,Uuid,Uuid,i32,i32)> = sqlx::query_as("SELECT id, guild_a, guild_b, score_a, score_b FROM guild_wars WHERE guild_a=$1 OR guild_b=$1 ORDER BY created_at DESC LIMIT 10").bind(gid).fetch_all(&state.db).await?;
        Ok(HttpResponse::Ok().json(wars.into_iter().map(|(id,a,b,sa,sb)| serde_json::json!({"id":id,"guild_a":a,"guild_b":b,"score_a":sa,"score_b":sb})).collect::<Vec<_>>()))
    } else {
        Ok(HttpResponse::Ok().json(Vec::<serde_json::Value>::new()))
    }
}

async fn territories(state: web::Data<Arc<AppState>>, _user: AuthenticatedUser) -> AppResult<HttpResponse> {
    let rows: Vec<(String,Option<Uuid>,serde_json::Value)> = sqlx::query_as("SELECT name, owner_guild, buff FROM guild_territories ORDER BY name").fetch_all(&state.db).await?;
    Ok(HttpResponse::Ok().json(rows.into_iter().map(|(name,owner,buff)| serde_json::json!({"name":name,"owner_guild":owner,"buff":buff})).collect::<Vec<_>>()))
}
