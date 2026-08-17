use std::{env, fs};

use crate::error::{AppError, AppResult};

#[derive(Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub jwt_private_key: Vec<u8>,
    pub jwt_public_key: Vec<u8>,
    pub access_token_minutes: i64,
    pub refresh_token_days: i64,
    pub allowed_origin: String,
}

impl Config {
    pub fn from_env() -> AppResult<Self> {
        dotenvy::dotenv().ok();
        let private_path = required("JWT_PRIVATE_KEY_PATH")?;
        let public_path = required("JWT_PUBLIC_KEY_PATH")?;

        Ok(Self {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: parse_or("PORT", 8080)?,
            database_url: required("DATABASE_URL")?,
            redis_url: required("REDIS_URL")?,
            jwt_private_key: fs::read(&private_path).map_err(|e| {
                AppError::Config(format!("não foi possível ler {private_path}: {e}"))
            })?,
            jwt_public_key: fs::read(&public_path).map_err(|e| {
                AppError::Config(format!("não foi possível ler {public_path}: {e}"))
            })?,
            access_token_minutes: parse_or("ACCESS_TOKEN_MINUTES", 15)?,
            refresh_token_days: parse_or("REFRESH_TOKEN_DAYS", 30)?,
            allowed_origin: env::var("ALLOWED_ORIGIN").unwrap_or_else(|_| "http://localhost:3000".into()),
        })
    }
}

fn required(name: &str) -> AppResult<String> {
    env::var(name).map_err(|_| AppError::Config(format!("variável obrigatória ausente: {name}")))
}

fn parse_or<T>(name: &str, default: T) -> AppResult<T>
where
    T: std::str::FromStr + ToString,
    T::Err: std::fmt::Display,
{
    env::var(name)
        .unwrap_or_else(|_| default.to_string())
        .parse()
        .map_err(|e| AppError::Config(format!("{name} inválida: {e}")))
}
