use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;
use sqlx::SqlitePool;
use serenity::{all::{Cache, Http}, prelude::*};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub discord: DiscordConfig,
    pub web: WebConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordConfig {
    pub token: String,
    pub application_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfig {
    pub host: String,
    pub port: u16,
    pub base_url: String,
    pub oauth: OAuthConfig,
    pub embed: EmbedConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedConfig {
    pub directory: String,
    pub max_age_hours: u64,
    pub cleanup_interval_hours: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();
        
        let discord_token = env::var("DISCORD_TOKEN")
            .map_err(|_| anyhow::anyhow!("DISCORD_TOKEN environment variable not set"))?;
        
        let application_id = env::var("DISCORD_APPLICATION_ID")
            .map_err(|_| anyhow::anyhow!("DISCORD_APPLICATION_ID environment variable not set"))?
            .parse::<u64>()
            .map_err(|_| anyhow::anyhow!("Invalid DISCORD_APPLICATION_ID format"))?;
        
        let oauth_client_id = env::var("DISCORD_CLIENT_ID")
            .map_err(|_| anyhow::anyhow!("DISCORD_CLIENT_ID environment variable not set"))?;
        
        let oauth_client_secret = env::var("DISCORD_CLIENT_SECRET")
            .map_err(|_| anyhow::anyhow!("DISCORD_CLIENT_SECRET environment variable not set"))?;
        
        let base_url = env::var("BASE_URL")
            .unwrap_or_else(|_| "http://localhost:3000".to_string());
        
        let host = env::var("HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_string());
        
        let port = env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .unwrap_or(3000);
        
        let redirect_uri = format!("{}/auth/callback", base_url);
        
        Ok(Config {
            discord: DiscordConfig {
                token: discord_token,
                application_id,
            },
            web: WebConfig {
                host,
                port,
                base_url,
                oauth: OAuthConfig {
                    client_id: oauth_client_id,
                    client_secret: oauth_client_secret,
                    redirect_uri,
                },
                embed: EmbedConfig {
                    directory: "embed_files".to_string(),
                    max_age_hours: 24,
                    cleanup_interval_hours: 6,
                },
            },
            database: DatabaseConfig {
                url: "data/db.sqlite".to_string(),
            },
        })
    }
    
    #[cfg(test)]
    pub fn test_config() -> Self {
        Self {
            discord: DiscordConfig {
                token: "test_token".to_string(),
                application_id: 12345,
            },
            web: WebConfig {
                host: "127.0.0.1".to_string(),
                port: 3000,
                base_url: "http://localhost:3000".to_string(),
                oauth: OAuthConfig {
                    client_id: "test_client_id".to_string(),
                    client_secret: "test_client_secret".to_string(),
                    redirect_uri: "http://localhost:3000/auth/callback".to_string(),
                },
                embed: EmbedConfig {
                    directory: "test_embed_files".to_string(),
                    max_age_hours: 24,
                    cleanup_interval_hours: 6,
                },
            },
            database: DatabaseConfig {
                url: ":memory:".to_string(),
            },
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: Arc<SqlitePool>,
    pub cache: Arc<Cache>,
    pub http: Arc<Http>,
}

impl AppState {
    pub fn new(
        config: Arc<Config>,
        db: Arc<SqlitePool>,
        cache: Arc<Cache>,
        http: Arc<Http>,
    ) -> Self {
        Self {
            config,
            db,
            cache,
            http,
        }
    }
}