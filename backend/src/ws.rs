use std::{sync::Arc, time::{Duration, Instant}};

use actix::{Actor, ActorContext, AsyncContext, StreamHandler};
use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{auth::middleware::AuthenticatedUser, combat::waves::WaveEvent, error::{AppError, AppResult}, state::AppState};

const HEARTBEAT: Duration = Duration::from_secs(15);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(45);
const PROTOCOL_VERSION: u8 = 1;

#[derive(Deserialize)]
pub struct ResumeQuery {
    #[serde(default)]
    pub after_sequence: u32,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
enum ServerMessage {
    Welcome { version: u8, combat_id: Uuid, sequence: u32 },
    CombatState { version: u8, combat_id: Uuid, sequence: u32, event: WaveEvent },
    Heartbeat { version: u8, sequence: u32 },
    Error { version: u8, code: &'static str },
}

/// Upgrade autenticado. `after_sequence` permite retomar eventos já persistidos da sessão.
pub async fn combat_stream(
    request: HttpRequest,
    payload: web::Payload,
    state: web::Data<Arc<AppState>>,
    user: AuthenticatedUser,
    combat_id: web::Path<Uuid>,
    query: web::Query<ResumeQuery>,
) -> AppResult<HttpResponse> {
    if state.config.allowed_origin != "*" {
        if let Some(origin) = request.headers().get("Origin").and_then(|value| value.to_str().ok()) {
            if origin != state.config.allowed_origin {
                return Err(AppError::Unauthorized);
            }
        }
    }
    let combat_id = combat_id.into_inner();
    let events: Option<serde_json::Value> = sqlx::query_scalar(
        "SELECT events FROM combat_sessions WHERE id=$1 AND user_id=$2 AND status='resolved'",
    )
    .bind(combat_id)
    .bind(user.user_id)
    .fetch_optional(&state.db)
    .await?;
    let events = events.ok_or(AppError::NotFound)?;
    let events: Vec<WaveEvent> = serde_json::from_value(events)
        .map_err(|error| AppError::Internal(format!("eventos de combate inválidos: {error}")))?;
    let pending = events.into_iter().filter(|event| event.sequence > query.after_sequence).collect();
    ws::start(CombatSocket::new(combat_id, pending), &request, payload)
        .map_err(|error| AppError::Internal(format!("websocket: {error}")))
}

struct CombatSocket {
    combat_id: Uuid,
    events: Vec<WaveEvent>,
    last_heartbeat: Instant,
}

impl CombatSocket {
    fn new(combat_id: Uuid, events: Vec<WaveEvent>) -> Self {
        Self { combat_id, events, last_heartbeat: Instant::now() }
    }

    fn send(ctx: &mut ws::WebsocketContext<Self>, message: ServerMessage) {
        match serde_json::to_string(&message) {
            Ok(payload) => ctx.text(payload),
            Err(error) => {
                log::error!("falha ao serializar evento WebSocket: {error}");
                ctx.stop();
            }
        }
    }
}

impl Actor for CombatSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        Self::send(ctx, ServerMessage::Welcome { version: PROTOCOL_VERSION, combat_id: self.combat_id, sequence: 0 });
        for event in self.events.clone() {
            Self::send(ctx, ServerMessage::CombatState { version: PROTOCOL_VERSION, combat_id: self.combat_id, sequence: event.sequence, event });
        }
        ctx.run_interval(HEARTBEAT, |socket, ctx| {
            if Instant::now().duration_since(socket.last_heartbeat) > CLIENT_TIMEOUT {
                ctx.stop();
                return;
            }
            Self::send(ctx, ServerMessage::Heartbeat { version: PROTOCOL_VERSION, sequence: 0 });
            ctx.ping(b"");
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for CombatSocket {
    fn handle(&mut self, message: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match message {
            Ok(ws::Message::Ping(payload)) => { self.last_heartbeat = Instant::now(); ctx.pong(&payload); }
            Ok(ws::Message::Pong(_)) => self.last_heartbeat = Instant::now(),
            Ok(ws::Message::Text(text)) if text == "HEARTBEAT" => self.last_heartbeat = Instant::now(),
            Ok(ws::Message::Close(reason)) => { ctx.close(reason); ctx.stop(); }
            Ok(ws::Message::Binary(_)) | Ok(ws::Message::Continuation(_)) => Self::send(ctx, ServerMessage::Error { version: PROTOCOL_VERSION, code: "UNSUPPORTED_MESSAGE" }),
            Ok(ws::Message::Nop) => {},
            Err(error) => { log::debug!("erro de protocolo WebSocket: {error}"); ctx.stop(); }
            _ => Self::send(ctx, ServerMessage::Error { version: PROTOCOL_VERSION, code: "INVALID_MESSAGE" }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn protocolo_serializa_com_versao_e_sequencia() {
        let message = ServerMessage::Heartbeat { version: 1, sequence: 7 };
        let value = serde_json::to_value(message).unwrap();
        assert_eq!(value["type"], "HEARTBEAT");
        assert_eq!(value["version"], 1);
        assert_eq!(value["sequence"], 7);
    }
}
