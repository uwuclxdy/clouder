pub use config::{AppState, Config};
pub use database::selfroles::{SelfRoleConfig, SelfRoleRole};
pub use database::welcome_goodbye::WelcomeGoodbyeConfig;
pub use shared::models::{
    ChannelInfo, CreateSelfRoleRequest, RoleInfo, SelfRoleData, UserPermissions,
};
pub use shared::{
    create_selfrole, delete_selfrole, get_guild_channels, get_guild_roles, list_selfroles,
};

mod commands;
mod config;
mod database;
mod events;
mod logging;
mod shared;
mod utils;

pub use crate::commands::about::about;
pub use crate::commands::help::help;
pub use crate::commands::mediaonly::mediaonly;
pub use crate::commands::purge::purge;
pub use crate::commands::selfroles::selfroles;
pub use crate::database::selfroles::SelfRoleCooldown;
pub use crate::events::event_handler;
pub use crate::logging::{debug, error, info};

use anyhow::Result;
use poise::serenity_prelude as serenity;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tokio::try_join;

type Data = AppState;
type Error = Box<dyn std::error::Error + Send + Sync>;

pub fn run() -> Result<()> {
    tokio::runtime::Runtime::new()?.block_on(async_main())
}

async fn async_main() -> Result<()> {
    logging::init();

    info!("starting clouder");

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

    info!("starting discord client");

    // Start Discord bot and web dashboard
    try_join!(
        async {
            client.start().await.map_err(anyhow::Error::msg)?;
            Ok::<(), anyhow::Error>(())
        },
        async {
            #[cfg(feature = "web")]
            {
                clouder_web::run().await.map_err(|e| {
                    error!("web dashboard error: {}", e);
                    e
                })?;
            }
            Ok::<(), anyhow::Error>(())
        }
    )?;

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
