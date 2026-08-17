use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("configuração inválida: {0}")]
    Config(String),
    #[error("credenciais inválidas")]
    InvalidCredentials,
    #[error("não autorizado")]
    Unauthorized,
    #[error("conflito: {0}")]
    Conflict(String),
    #[error("requisição inválida: {0}")]
    Validation(String),
    #[error("recurso não encontrado")]
    NotFound,
    #[error("erro interno")]
    Internal(String),
}

#[derive(Serialize)]
struct ErrorBody<'a> {
    code: &'a str,
    message: String,
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidCredentials | Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Config(_) | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let code = match self {
            Self::Config(_) | Self::Internal(_) => "INTERNAL_ERROR",
            Self::InvalidCredentials => "INVALID_CREDENTIALS",
            Self::Unauthorized => "UNAUTHORIZED",
            Self::Conflict(_) => "CONFLICT",
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::NotFound => "NOT_FOUND",
        };
        let message = match self {
            // Não vaza detalhes de infraestrutura para o cliente.
            Self::Config(_) | Self::Internal(_) => "O servidor não conseguiu concluir a operação".into(),
            other => other.to_string(),
        };
        HttpResponse::build(self.status_code()).json(ErrorBody { code, message })
    }
}

impl From<sqlx::Error> for AppError {
    fn from(value: sqlx::Error) -> Self {
        log::error!("erro de banco: {value}");
        Self::Internal("database".into())
    }
}

impl From<redis::RedisError> for AppError {
    fn from(value: redis::RedisError) -> Self {
        log::error!("erro do Redis: {value}");
        Self::Internal("redis".into())
    }
}
