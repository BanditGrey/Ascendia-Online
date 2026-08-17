use sqlx::PgPool;

use crate::{auth::token::TokenService, config::Config, error::AppResult};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: redis::Client,
    pub tokens: TokenService,
    pub config: Config,
}

impl AppState {
    pub async fn connect(config: Config) -> AppResult<Self> {
        let db = sqlx::postgres::PgPoolOptions::new()
            .max_connections(20)
            .connect(&config.database_url)
            .await?;
        sqlx::migrate!("./migrations").run(&db).await.map_err(|e| {
            log::error!("falha nas migrations: {e}");
            crate::error::AppError::Internal("migration".into())
        })?;
        let redis = redis::Client::open(config.redis_url.as_str())?;
        let tokens = TokenService::new(&config)?;
        Ok(Self { db, redis, tokens, config })
    }
}
