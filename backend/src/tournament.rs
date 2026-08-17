use std::sync::Arc;
use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::{auth::middleware::AuthenticatedUser, error::{AppError, AppResult}, state::AppState};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/tournament")
        .route("/status", web::get().to(status))
        .route("/register", web::post().to(register))
        .route("/bracket", web::get().to(bracket))
    );
}

async fn status(state: web::Data<Arc<AppState>>, _user: AuthenticatedUser) -> AppResult<HttpResponse> {
    let tour: Option<(Uuid,String,String,chrono::DateTime<chrono::Utc>)> = sqlx::query_as("SELECT id, name, status, created_at FROM tournaments ORDER BY created_at DESC LIMIT 1").fetch_optional(&state.db).await?;
    if let Some((id,name,status,_)) = tour {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tournament_participants WHERE tournament_id=$1").bind(id).fetch_one(&state.db).await?;
        Ok(HttpResponse::Ok().json(serde_json::json!({"id":id,"name":name,"status":status,"participants":count,"max":32,"next_thursday":"Quinta 20h"})))
    } else {
        // Cria torneio da semana se não existe
        let id = Uuid::new_v4();
        sqlx::query("INSERT INTO tournaments (id, name, status) VALUES ($1,'Torneio Semanal — 32', 'registration') ON CONFLICT DO NOTHING").bind(id).execute(&state.db).await?;
        Ok(HttpResponse::Ok().json(serde_json::json!({"id":id,"name":"Torneio Semanal — 32","status":"registration","participants":0})))
    }
}

async fn register(state: web::Data<Arc<AppState>>, user: AuthenticatedUser) -> AppResult<HttpResponse> {
    let tour_id: Uuid = sqlx::query_scalar("SELECT id FROM tournaments WHERE status='registration' ORDER BY created_at DESC LIMIT 1").fetch_optional(&state.db).await?.ok_or(AppError::Validation("sem torneio em registro (quarta)".into()))?;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tournament_participants WHERE tournament_id=$1").bind(tour_id).fetch_one(&state.db).await?;
    if count >= 32 { return Err(AppError::Validation("bracket cheio 32".into())); }
    let already: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM tournament_participants WHERE tournament_id=$1 AND user_id=$2)").bind(tour_id).bind(user.user_id).fetch_one(&state.db).await?;
    if already { return Err(AppError::Conflict("já registrado".into())); }
    sqlx::query("INSERT INTO tournament_participants (tournament_id, user_id, seed) VALUES ($1,$2,$3)").bind(tour_id).bind(user.user_id).bind(rand::random::<i32>()).execute(&state.db).await?;
    Ok(HttpResponse::Created().json(serde_json::json!({"registered":tour_id})))
}

async fn bracket(state: web::Data<Arc<AppState>>, _user: AuthenticatedUser) -> AppResult<HttpResponse> {
    let tour_id: Option<Uuid> = sqlx::query_scalar("SELECT id FROM tournaments ORDER BY created_at DESC LIMIT 1").fetch_optional(&state.db).await?;
    if let Some(tid) = tour_id {
        let participants: Vec<(Uuid,i32)> = sqlx::query_as("SELECT user_id, seed FROM tournament_participants WHERE tournament_id=$1 ORDER BY seed").bind(tid).fetch_all(&state.db).await?;
        let matches: Vec<(Uuid,i32,Option<Uuid>,Option<Uuid>)> = sqlx::query_as("SELECT id, round, player_a, player_b FROM tournament_matches WHERE tournament_id=$1 ORDER BY round, created_at").bind(tid).fetch_all(&state.db).await.unwrap_or_default();
        Ok(HttpResponse::Ok().json(serde_json::json!({"tournament_id":tid,"participants":participants.into_iter().map(|(uid,seed)| serde_json::json!({"user_id":uid,"seed":seed})).collect::<Vec<_>>(),"matches":matches.into_iter().map(|(id,r,a,b)| serde_json::json!({"id":id,"round":r,"player_a":a,"player_b":b})).collect::<Vec<_>>() })))
    } else {
        Ok(HttpResponse::Ok().json(serde_json::json!({"participants":[],"matches":[]})))
    }
}
