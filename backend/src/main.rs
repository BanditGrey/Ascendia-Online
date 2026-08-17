mod admin;
mod arena;
mod auth;
mod auction;
mod awakening;
mod battle_pass;
mod chat;
mod combat;
mod config;
mod cosmetics;
mod crafting;
mod dungeon;
mod enchant;
mod error;
mod events;
mod expedition;
mod friends;
mod guild;
mod guild_war;
mod http;
mod inventory;
mod marketplace;
mod oauth;
mod offline_rewards;
mod player;
mod quests;
mod raid;
mod ranking;
mod rate_limit;
mod runes;
mod skills;
mod squad;
mod state;
mod totp;
mod tournament;
mod tower;
mod trade;
mod vip;
mod world_boss;
mod ws;

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
            .route("/metrics", web::get().to(http::health::health))
            .service(web::scope("/api/v1")
                .configure(auth::routes::configure)
                .configure(oauth::configure)
                .configure(totp::configure)
                .configure(chat::configure)
                .configure(combat::routes::configure)
                .configure(cosmetics::configure)
                .configure(inventory::routes::configure)
                .configure(offline_rewards::configure)
                .configure(player::routes::configure)
                .configure(ranking::configure)
                .configure(vip::configure)
                .configure(battle_pass::configure)
                .configure(guild::configure)
                .configure(guild_war::configure)
                .configure(marketplace::configure)
                .configure(auction::configure)
                .configure(trade::configure)
                .configure(runes::configure)
                .configure(crafting::configure)
                .configure(skills::configure)
                .configure(tower::configure)
                .configure(arena::configure)
                .configure(dungeon::configure)
                .configure(friends::configure)
                .configure(quests::configure)
                .configure(tournament::configure)
                .configure(awakening::configure)
                .configure(expedition::configure)
                .configure(world_boss::configure)
                .configure(raid::configure)
                .configure(enchant::configure)
                .configure(events::configure)
                .configure(admin::configure)
                .route("/ws/combat/{combat_id}", web::get().to(ws::combat_stream)))
    })
    .bind(bind).map_err(|e| error::AppError::Config(format!("bind: {e}")))?
    .run().await.map_err(|e| error::AppError::Internal(format!("servidor: {e}")))
}
