mod auth;
mod combat;
mod config;
mod error;
mod http;
mod state;

use std::sync::Arc;
use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use config::Config;
use error::AppResult;
use state::AppState;

#[actix_web::main]
async fn main() -> AppResult<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("ascendia_server=info,actix_web=info"));
    let config = Config::from_env()?;
    let bind = (config.host.clone(), config.port);
    let allowed_origin = config.allowed_origin.clone();
    let state = Arc::new(AppState::connect(config).await?);
    log::info!("Ascendia API iniciando em {}:{}", bind.0, bind.1);

    HttpServer::new(move || {
        let cors = if allowed_origin == "*" { Cors::permissive() } else {
            Cors::default().allowed_origin(&allowed_origin).allowed_methods(vec!["GET","POST","PUT","DELETE"]).allowed_headers(vec![actix_web::http::header::AUTHORIZATION, actix_web::http::header::CONTENT_TYPE]).max_age(3600)
        };
        App::new()
            .app_data(web::Data::new(state.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            .route("/health", web::get().to(http::health::health))
            .service(web::scope("/api/v1").configure(auth::routes::configure).configure(combat::routes::configure))
    })
    .bind(bind).map_err(|e| error::AppError::Config(format!("bind: {e}")))?
    .run().await.map_err(|e| error::AppError::Internal(format!("servidor: {e}")))
}
