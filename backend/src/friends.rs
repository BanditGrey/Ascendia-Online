use std::sync::Arc;
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{auth::middleware::AuthenticatedUser, error::{AppError, AppResult}, state::AppState};

#[derive(FromRow, Serialize)]
struct FriendView { user_id: Uuid, display_name: String, level: i16, power_rating: i64 }
#[derive(Deserialize)]
struct RequestBody { to_user_id: Uuid }
#[derive(Deserialize)]
struct AcceptBody { request_id: Uuid }

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/friends")
        .route("", web::get().to(list))
        .route("/requests", web::get().to(list_requests))
        .route("/request", web::post().to(request))
        .route("/accept", web::post().to(accept))
        .route("/{friend_id}", web::delete().to(remove))
    );
}

async fn list(state: web::Data<Arc<AppState>>, user: AuthenticatedUser) -> AppResult<HttpResponse> {
    let friends: Vec<FriendView> = sqlx::query_as("SELECT f.friend_user_id as user_id, u.display_name, c.level, cs.power_rating FROM friendships f JOIN users u ON u.id=f.friend_user_id JOIN characters c ON c.user_id=u.id AND c.is_leader=true JOIN character_stats cs ON cs.character_id=c.id WHERE f.user_id=$1 LIMIT 100")
        .bind(user.user_id).fetch_all(&state.db).await?;
    Ok(HttpResponse::Ok().json(friends))
}

async fn list_requests(state: web::Data<Arc<AppState>>, user: AuthenticatedUser) -> AppResult<HttpResponse> {
    let reqs: Vec<(Uuid,Uuid,String)> = sqlx::query_as("SELECT r.id, r.from_user_id, u.display_name FROM friend_requests r JOIN users u ON u.id=r.from_user_id WHERE r.to_user_id=$1 AND r.status='pending' ORDER BY r.created_at DESC")
        .bind(user.user_id).fetch_all(&state.db).await?;
    let out: Vec<serde_json::Value> = reqs.into_iter().map(|(id,from,name)| serde_json::json!({"request_id":id,"from_user_id":from,"display_name":name})).collect();
    Ok(HttpResponse::Ok().json(out))
}

async fn request(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, body: web::Json<RequestBody>) -> AppResult<HttpResponse> {
    if user.user_id == body.to_user_id { return Err(AppError::Validation("não pode adicionar a si mesmo".into())); }
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM friendships WHERE user_id=$1 AND friend_user_id=$2)").bind(user.user_id).bind(body.to_user_id).fetch_one(&state.db).await?;
    if exists { return Err(AppError::Conflict("já são amigos".into())); }
    let already: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM friend_requests WHERE from_user_id=$1 AND to_user_id=$2 AND status='pending')").bind(user.user_id).bind(body.to_user_id).fetch_one(&state.db).await?;
    if already { return Err(AppError::Conflict("pedido já enviado".into())); }
    let target_exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id=$1)").bind(body.to_user_id).fetch_one(&state.db).await?;
    if !target_exists { return Err(AppError::NotFound); }
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM friendships WHERE user_id=$1").bind(user.user_id).fetch_one(&state.db).await?;
    if count >= 100 { return Err(AppError::Validation("limite 100 amigos".into())); }
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO friend_requests (id, from_user_id, to_user_id) VALUES ($1,$2,$3)").bind(id).bind(user.user_id).bind(body.to_user_id).execute(&state.db).await?;
    Ok(HttpResponse::Created().json(serde_json::json!({"request_id":id})))
}

async fn accept(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, body: web::Json<AcceptBody>) -> AppResult<HttpResponse> {
    let mut tx = state.db.begin().await?;
    let row: Option<(Uuid,Uuid)> = sqlx::query_as("SELECT from_user_id, to_user_id FROM friend_requests WHERE id=$1 AND status='pending' FOR UPDATE").bind(body.request_id).fetch_optional(&mut *tx).await?;
    let (from,to) = row.ok_or(AppError::NotFound)?;
    if to != user.user_id { return Err(AppError::Unauthorized); }
    sqlx::query("UPDATE friend_requests SET status='accepted' WHERE id=$1").bind(body.request_id).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO friendships (user_id, friend_user_id) VALUES ($1,$2) ON CONFLICT DO NOTHING").bind(from).bind(to).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO friendships (user_id, friend_user_id) VALUES ($1,$2) ON CONFLICT DO NOTHING").bind(to).bind(from).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"accepted":body.request_id})))
}

async fn remove(state: web::Data<Arc<AppState>>, user: AuthenticatedUser, friend_id: web::Path<Uuid>) -> AppResult<HttpResponse> {
    let fid = friend_id.into_inner();
    sqlx::query("DELETE FROM friendships WHERE (user_id=$1 AND friend_user_id=$2) OR (user_id=$2 AND friend_user_id=$1)").bind(user.user_id).bind(fid).execute(&state.db).await?;
    Ok(HttpResponse::NoContent().finish())
}
