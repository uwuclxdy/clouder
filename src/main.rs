mod config;
mod database;
mod commands;
mod events;
mod web;
mod utils;

#[cfg(test)]
mod tests;

use crate::config::{Config, AppState};
use crate::commands::selfroles::selfroles;
use crate::commands::video::{video, video_help, cleanup_embeds};
use crate::commands::about::about;
use crate::database::selfroles::SelfRoleCooldown;
use anyhow::Result;
use poise::serenity_prelude as serenity;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{error, info};

// Bot data for Poise
type Data = AppState;
type Error = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting Clouder Discord Bot...");

    // Load configuration
    let config = Arc::new(Config::from_env()?);
    info!("Configuration loaded successfully");

    // Initialize database
    let db = database::initialize_database(&config.database.url).await?;
    info!("Database initialized successfully");

    // Setup Discord client
    let token = config.discord.token.clone();
    let intents = serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::GUILDS
        | serenity::GatewayIntents::MESSAGE_CONTENT;

    let config_clone = config.clone();
    let db_clone = db.clone();
    let token_clone = token.clone();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![selfroles(), video(), video_help(), cleanup_embeds(), about()],
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

                // Create shared app state
                let http = Arc::new(serenity::Http::new(&token));
                let cache = Arc::new(serenity::all::Cache::new());

                let app_state = AppState::new(config, Arc::new(db), cache, http);

                Ok(app_state)
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await?;

    let mut client = client;

    // Get the app state from the framework data after setup
    let cache = client.cache.clone();
    let http = client.http.clone();
    let app_state = AppState::new(config.clone(), Arc::new(db), cache, http);

    // Start cleanup tasks
    start_cleanup_task(app_state.clone());
    start_embed_cleanup_task(app_state.clone());

    // Start web server
    let web_config = config.web.clone();
    let web_state = app_state.clone();
    tokio::spawn(async move {
        if let Err(e) = start_web_server(web_config, web_state).await {
            error!("Web server error: {}", e);
        }
    });

    // Start Discord client
    info!("Starting Discord client...");
    if let Err(e) = client.start().await {
        error!("Discord client error: {}", e);
    }

    Ok(())
}

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            info!("Bot {} is ready!", data_about_bot.user.name);
        }
        serenity::FullEvent::InteractionCreate { interaction } => {
            events::handle_interaction_create(ctx, interaction, data).await;
        }
        serenity::FullEvent::MessageDelete { channel_id, deleted_message_id, guild_id } => {
            events::handle_message_delete(ctx, channel_id, deleted_message_id, guild_id, data).await;
        }
        _ => {}
    }
    Ok(())
}

async fn start_web_server(
    web_config: config::WebConfig,
    app_state: AppState,
) -> Result<()> {
    let app = web::create_router(app_state);
    let addr = format!("{}:{}", web_config.host, web_config.port);

    info!("Starting web server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn start_cleanup_task(app_state: AppState) {
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(300)).await; // Run every 5 minutes

            if let Err(e) = SelfRoleCooldown::cleanup_expired(&app_state.db).await {
                error!("Failed to cleanup expired cooldowns: {}", e);
            } else {
                tracing::debug!("Cleaned up expired cooldowns");
            }
        }
    });
}

fn start_embed_cleanup_task(app_state: AppState) {
    let embed_config = app_state.config.web.embed.clone();

    // Check if cleanup is disabled (either value set to 0)
    if embed_config.cleanup_interval_hours == 0 || embed_config.max_age_hours == 0 {
        info!("Embed cleanup disabled (cleanup_interval_hours={}, max_age_hours={})",
              embed_config.cleanup_interval_hours, embed_config.max_age_hours);
        return;
    }

    tokio::spawn(async move {
        loop {
            let interval_seconds = embed_config.cleanup_interval_hours * 3600;
            sleep(Duration::from_secs(interval_seconds)).await;

            info!("Running automatic embed cleanup...");
            match utils::embed::cleanup_old_embeds(
                &embed_config.directory,
                embed_config.max_age_hours
            ).await {
                Ok(cleaned_count) => {
                    if cleaned_count > 0 {
                        info!("Automatic cleanup completed: {} files removed", cleaned_count);
                    } else {
                        tracing::debug!("No old embed files to clean up");
                    }
                }
                Err(e) => {
                    error!("Automatic embed cleanup failed: {}", e);
                }
            }
        }
    });
}
