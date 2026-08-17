use actix_web::{dev::Payload, web, FromRequest, HttpRequest};
use std::{future::{ready, Ready}, sync::Arc};
use uuid::Uuid;

use crate::{error::AppError, state::AppState};

#[derive(Clone, Copy, Debug)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub session_id: Uuid,
}

impl FromRequest for AuthenticatedUser {
    type Error = AppError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let result = (|| {
            let state = req.app_data::<web::Data<Arc<AppState>>>().ok_or(AppError::Unauthorized)?;
            let raw = req.headers().get("Authorization").and_then(|v| v.to_str().ok())
                .and_then(|v| v.strip_prefix("Bearer ")).ok_or(AppError::Unauthorized)?;
            let claims = state.tokens.verify(raw)?;
            Ok(Self { user_id: claims.sub, session_id: claims.sid })
        })();
        ready(result)
    }
}
