use std::sync::Arc;

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{auth::middleware::AuthenticatedUser, error::{AppError, AppResult}, state::AppState};

#[derive(FromRow, Serialize)]
struct Guild { id: Uuid, name: String, level: i16, leader_user_id: Uuid, member_count: i16 }

#[derive(Deserialize)]
struct CreateGuild { name: String }

#[derive(Deserialize)]
struct JoinGuild { guild_id: Uuid }

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/guilds")
        .route("", web::get().to(list))
        .route("", web::post().to(create))
        .route("/join", web::post().to(join))
        .route("/leave", web::post().to(leave))
        .route("/me", web::get().to(my_guild))
    );
}

async fn list(state: web::Data<Arc<AppState>>, _user: AuthenticatedUser) -> AppResult<HttpResponse> {
    let guilds: Vec<Guild> = sqlx::query_as("SELECT id,name,level,leader_user_id,member_count FROM guilds ORDER BY level DESC, name LIMIT 50")
        .fetch_all(&state.db).await?;
    Ok(HttpResponse::Ok().json(guilds))
}

async fn my_guild(state: web::Data<Arc<AppState>>, user: AuthenticatedUser) -> AppResult<HttpResponse> {
    let guild: Option<Guild> = sqlx::query_as("SELECT g.id,g.name,g.level,g.leader_user_id,g.member_count FROM guilds g JOIN guild_members m ON m.guild_id=g.id WHERE m.user_id=$1")
        .bind(user.user_id).fetch_optional(&state.db).await?;
    if let Some(g) = guild { Ok(HttpResponse::Ok().json(g)) } else { Ok(HttpResponse::Ok().json(serde_json::json!(null))) }
}

async fn create(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, body: web::Json<CreateGuild>) -> AppResult<HttpResponse> {
    let name = body.name.trim();
    if name.len() < 3 || name.len() > 40 { return Err(AppError::Validation("nome da guilda deve ter 3-40 caracteres".into())); }
    // Custo 1000 Gold + level 20
    let mut tx = state.db.begin().await?;
    let (level, gold): (i16, i64) = sqlx::query_as("SELECT c.level, u.gold FROM characters c JOIN users u ON u.id=c.user_id WHERE c.user_id=$1 AND c.is_leader=true FOR UPDATE OF u")
        .bind(user.user_id).fetch_one(&mut *tx).await?;
    if level < 20 { return Err(AppError::Validation("level 20 necessário para criar guilda".into())); }
    if gold < 1000 { return Err(AppError::Validation("1000 Gold necessários".into())); }
    let already: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM guild_members WHERE user_id=$1)").bind(user.user_id).fetch_one(&mut *tx).await?;
    if already { return Err(AppError::Conflict("você já está em uma guilda".into())); }
    let guild_id = Uuid::new_v4();
    sqlx::query("UPDATE users SET gold=gold-1000 WHERE id=$1").bind(user.user_id).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO guilds (id,name,leader_user_id) VALUES ($1,$2,$3)").bind(guild_id).bind(name).bind(user.user_id).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO guild_members (guild_id,user_id,role) VALUES ($1,$2,'leader')").bind(guild_id).bind(user.user_id).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO audit_logs (actor_user_id,action,metadata) VALUES ($1,'GUILD_CREATED',$2)").bind(user.user_id).bind(serde_json::json!({"guild_id":guild_id,"name":name})).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(HttpResponse::Created().json(serde_json::json!({"guild_id":guild_id})))
}

async fn join(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, body: web::Json<JoinGuild>) -> AppResult<HttpResponse> {
    let mut tx = state.db.begin().await?;
    let already: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM guild_members WHERE user_id=$1)").bind(user.user_id).fetch_one(&mut *tx).await?;
    if already { return Err(AppError::Conflict("já está em guilda".into())); }
    let (count,): (i16,) = sqlx::query_as("SELECT member_count FROM guilds WHERE id=$1 FOR UPDATE").bind(body.guild_id).fetch_optional(&mut *tx).await?.ok_or(AppError::NotFound)?;
    if count >= 50 { return Err(AppError::Validation("guilda cheia (50)".into())); }
    sqlx::query("INSERT INTO guild_members (guild_id,user_id) VALUES ($1,$2)").bind(body.guild_id).bind(user.user_id).execute(&mut *tx).await?;
    sqlx::query("UPDATE guilds SET member_count=member_count+1 WHERE id=$1").bind(body.guild_id).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"joined":body.guild_id})))
}

async fn leave(state: web::Data<Arc<AppState>>, user: AuthenticatedUser) -> AppResult<HttpResponse> {
    let mut tx = state.db.begin().await?;
    let guild_id: Option<Uuid> = sqlx::query_scalar("SELECT guild_id FROM guild_members WHERE user_id=$1").bind(user.user_id).fetch_optional(&mut *tx).await?;
    let guild_id = guild_id.ok_or(AppError::Validation("você não está em guilda".into()))?;
    let role: String = sqlx::query_scalar("SELECT role FROM guild_members WHERE user_id=$1").bind(user.user_id).fetch_one(&mut *tx).await?;
    if role == "leader" { return Err(AppError::Validation("líder não pode sair; transfira liderança ou delete".into())); }
    sqlx::query("DELETE FROM guild_members WHERE user_id=$1").bind(user.user_id).execute(&mut *tx).await?;
    sqlx::query("UPDATE guilds SET member_count=GREATEST(1, member_count-1) WHERE id=$1").bind(guild_id).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(HttpResponse::NoContent().finish())
}
