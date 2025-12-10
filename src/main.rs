mod commands;
mod config;
mod database;
mod events;
mod external;
mod logging;
mod utils;
mod web;

#[cfg(test)]
mod tests;

use crate::commands::about::about;
use crate::commands::help::help;
use crate::commands::mediaonly::mediaonly;
use crate::commands::purge::purge;
use crate::commands::selfroles::selfroles;
use crate::config::{AppState, Config};
use crate::database::selfroles::SelfRoleCooldown;
use crate::events::event_handler;
use crate::logging::{debug, error, info};
use anyhow::Result;
use poise::serenity_prelude as serenity;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

// Bot data for Poise
type Data = AppState;
type Error = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() -> Result<()> {
    logging::init();

    info!("starting clouder");

    // Initialize bot start time for uptime calculation
    let _ = *crate::commands::about::BOT_START_TIME;

    let config = Arc::new(Config::from_env()?);
    info!("config loaded");

    let db = database::initialize_database(&config.database.url).await?;
    info!("db init ok");
    let token = config.discord.token.clone();
    let intents = serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::GUILDS
        | serenity::GatewayIntents::MESSAGE_CONTENT
        | serenity::GatewayIntents::GUILD_MEMBERS;

    let config_clone = config.clone();
    let db_clone = db.clone();
    let token_clone = token.clone();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![selfroles(), about(), help(), purge(), mediaonly()],
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            let config = config_clone.clone();
            let db = db_clone.clone();
            let token = token_clone.clone();
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                let http = Arc::new(serenity::Http::new(&token));

                let app_state = AppState::new(config.clone(), Arc::new(db.clone()), http);

                // Store the database pool and app state in the context data for member events
                {
                    let mut data = ctx.data.write().await;
                    data.insert::<events::member_events::Database>(Arc::new(db));
                    data.insert::<events::member_events::AppStateKey>(Arc::new(app_state.clone()));
                }

                Ok(app_state)
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await?;

    let mut client = client;

    let http = client.http.clone();
    let app_state = AppState::new(config.clone(), Arc::new(db), http);

    start_cleanup_task(app_state.clone());

    let web_config = config.web.clone();
    let web_state = app_state.clone();
    tokio::spawn(async move {
        if let Err(e) = start_web_server(web_config, web_state).await {
            error!("web server: {}", e);
        }
    });

    info!("starting discord client");
    if let Err(e) = client.start().await {
        error!("discord client: {}", e);
    }

    Ok(())
}

async fn start_web_server(web_config: config::WebConfig, app_state: AppState) -> Result<()> {
    let app = web::create_router(app_state);
    let addr = format!("{}:{}", web_config.host, web_config.port);

    info!("starting web server: {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn start_cleanup_task(app_state: AppState) {
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(300)).await;

            if let Err(e) = SelfRoleCooldown::cleanup_expired(&app_state.db).await {
                error!("cleanup expired cooldowns: {}", e);
            } else {
                debug!("cleaned expired cooldowns");
            }
        }
    });
}
