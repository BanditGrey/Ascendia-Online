use actix_web::{dev::Payload, web, FromRequest, HttpRequest};
use redis::AsyncCommands;
use std::{future::Future, pin::Pin, sync::Arc};
use uuid::Uuid;

use crate::{error::AppError, state::AppState};

#[derive(Clone, Copy, Debug)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub session_id: Uuid,
}

impl FromRequest for AuthenticatedUser {
    type Error = AppError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let state = req.app_data::<web::Data<Arc<AppState>>>().cloned();
        let raw = req.headers().get("Authorization").and_then(|value| value.to_str().ok()).and_then(|value| value.strip_prefix("Bearer ")).map(str::to_owned);
        Box::pin(async move {
            let state = state.ok_or(AppError::Unauthorized)?;
            let raw = raw.ok_or(AppError::Unauthorized)?;
            let claims = state.tokens.verify(&raw)?;
            let mut redis = state.redis.get_multiplexed_async_connection().await.map_err(|_| AppError::Unauthorized)?;
            let revoked: bool = redis.exists(format!("auth:revoked:{}", claims.sid)).await.map_err(|_| AppError::Unauthorized)?;
            if revoked { return Err(AppError::Unauthorized); }
            Ok(Self { user_id: claims.sub, session_id: claims.sid })
        })
    }
}
