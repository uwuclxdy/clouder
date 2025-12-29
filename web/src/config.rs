use anyhow::Result;
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use std::{env, sync::Arc};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub web: WebConfig,
    pub discord: DiscordConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfig {
    pub host: String,
    pub port: u16,
    pub base_url: String,
    pub oauth: OAuthConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordConfig {
    pub token: String,
    pub application_id: u64,
    pub bot_owner: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenv().ok();

        let discord_token =
            env::var("DISCORD_TOKEN").map_err(|_| anyhow::anyhow!("DISCORD_TOKEN not set"))?;
        let application_id = env::var("DISCORD_APPLICATION_ID")
            .unwrap_or_default()
            .parse::<u64>()
            .unwrap_or(0);
        let bot_owner = env::var("BOT_OWNER")
            .unwrap_or_default()
            .parse::<u64>()
            .unwrap_or(0);

        let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .unwrap_or(3000);
        let base_url = env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
        let client_id = env::var("DISCORD_CLIENT_ID")
            .map_err(|_| anyhow::anyhow!("DISCORD_CLIENT_ID not set"))?;
        let client_secret = env::var("DISCORD_CLIENT_SECRET")
            .map_err(|_| anyhow::anyhow!("DISCORD_CLIENT_SECRET not set"))?;
        let redirect_uri = format!("{}/auth/callback", base_url);

        let database_url =
            env::var("DATABASE_URL").unwrap_or_else(|_| "data/db.sqlite".to_string());

        Ok(Config {
            discord: DiscordConfig {
                token: discord_token,
                application_id,
                bot_owner,
            },
            web: WebConfig {
                host,
                port,
                base_url,
                oauth: OAuthConfig {
                    client_id,
                    client_secret,
                    redirect_uri,
                },
            },
            database: DatabaseConfig { url: database_url },
        })
    }
}
