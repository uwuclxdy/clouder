use anyhow::Result;
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use serenity::all::{Cache, Http};
use sqlx::SqlitePool;
use std::env;
use std::sync::Arc;
use tracing::{error, info, warn};

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
    pub default_color: u32,
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
    pub fn from_env() -> Result<Self, anyhow::Error> {
        if let Err(e) = dotenv() {
            warn!("Could not load .env file: {}. Continuing with system environment variables.", e);
        }

        let discord_token = match env::var("DISCORD_TOKEN") {
            Ok(token) => token,
            Err(_) => {
                error!("DISCORD_TOKEN environment variable not set - this is required for the bot to function");
                return Err(anyhow::anyhow!("DISCORD_TOKEN environment variable not set"));
            }
        };
        let application_id = match env::var("DISCORD_APPLICATION_ID") {
            Ok(id_str) => match id_str.parse::<u64>() {
                Ok(id) => id,
                Err(e) => {
                    error!("DISCORD_APPLICATION_ID has invalid format '{}': {}", id_str, e);
                    return Err(anyhow::anyhow!("Invalid DISCORD_APPLICATION_ID format"));
                }
            },
            Err(_) => {
                error!("DISCORD_APPLICATION_ID environment variable not set - this is required for Discord interactions");
                return Err(anyhow::anyhow!("DISCORD_APPLICATION_ID environment variable not set"));
            }
        };
        let oauth_client_id = match env::var("DISCORD_CLIENT_ID") {
            Ok(id) => id,
            Err(_) => {
                error!("DISCORD_CLIENT_ID environment variable not set - this is required for OAuth");
                return Err(anyhow::anyhow!("DISCORD_CLIENT_ID environment variable not set"));
            }
        };

        let oauth_client_secret = match env::var("DISCORD_CLIENT_SECRET") {
            Ok(secret) => secret,
            Err(_) => {
                error!("DISCORD_CLIENT_SECRET environment variable not set - this is required for OAuth");
                return Err(anyhow::anyhow!("DISCORD_CLIENT_SECRET environment variable not set"));
            }
        };
        let base_url = match env::var("BASE_URL") {
            Ok(url) => {
                info!("Using custom BASE_URL: {}", url);
                url
            },
            Err(_) => {
                info!("BASE_URL not set, using default: http://localhost:3000");
                "http://localhost:3000".to_string()
            }
        };

        let host = match env::var("HOST") {
            Ok(host) => {
                info!("Using custom HOST: {}", host);
                host
            },
            Err(_) => {
                info!("HOST not set, using default: 127.0.0.1");
                "127.0.0.1".to_string()
            }
        };

        let port = match env::var("PORT") {
            Ok(port_str) => match port_str.parse::<u16>() {
                Ok(port) => {
                    info!("Using custom PORT: {}", port);
                    port
                },
                Err(e) => {
                    warn!("PORT '{}' has invalid format: {}. Using default port 3000", port_str, e);
                    3000
                }
            },
            Err(_) => {
                info!("PORT not set, using default: 3000");
                3000
            }
        };
        let database_url = match env::var("DATABASE_URL") {
            Ok(url) => {
                info!("Using custom DATABASE_URL: {}", url);
                url
            },
            Err(_) => {
                info!("DATABASE_URL not set, using default: data/db.sqlite");
                "data/db.sqlite".to_string()
            }
        };
        let embed_directory = match env::var("EMBED_DIRECTORY") {
            Ok(dir) => {
                info!("Using custom EMBED_DIRECTORY: {}", dir);
                dir
            },
            Err(_) => {
                info!("EMBED_DIRECTORY not set, using default: embed_files");
                "embed_files".to_string()
            }
        };

        let embed_max_age_hours = match env::var("EMBED_MAX_AGE_HOURS") {
            Ok(hours_str) => match hours_str.parse::<u64>() {
                Ok(hours) => {
                    if hours == 0 {
                        info!("EMBED_MAX_AGE_HOURS set to 0 - embed cleanup disabled");
                    } else {
                        info!("Using custom EMBED_MAX_AGE_HOURS: {}", hours);
                    }
                    hours
                },
                Err(e) => {
                    warn!("EMBED_MAX_AGE_HOURS '{}' has invalid format: {}. Using default: 24", hours_str, e);
                    24
                }
            },
            Err(_) => {
                info!("EMBED_MAX_AGE_HOURS not set, using default: 24");
                24
            }
        };

        let embed_cleanup_interval_hours = match env::var("EMBED_CLEANUP_INTERVAL_HOURS") {
            Ok(hours_str) => match hours_str.parse::<u64>() {
                Ok(hours) => {
                    if hours == 0 {
                        info!("EMBED_CLEANUP_INTERVAL_HOURS set to 0 - embed cleanup disabled");
                    } else {
                        info!("Using custom EMBED_CLEANUP_INTERVAL_HOURS: {}", hours);
                    }
                    hours
                },
                Err(e) => {
                    warn!("EMBED_CLEANUP_INTERVAL_HOURS '{}' has invalid format: {}. Using default: 6", hours_str, e);
                    6
                }
            },
            Err(_) => {
                info!("EMBED_CLEANUP_INTERVAL_HOURS not set, using default: 6");
                6
            }
        };

        let embed_default_color = match env::var("EMBED_DEFAULT_COLOR") {
            Ok(color_str) => {
                let color_str = color_str.trim();
                let parsed_color = if color_str.starts_with('#') {
                    u32::from_str_radix(&color_str[1..], 16)
                } else if color_str.starts_with("0x") || color_str.starts_with("0X") {
                    u32::from_str_radix(&color_str[2..], 16)
                } else {
                    color_str.parse::<u32>()
                };

                match parsed_color {
                    Ok(color) => {
                        info!("Using custom EMBED_DEFAULT_COLOR: {:#06X}", color);
                        color
                    },
                    Err(_) => {
                        0xFFFFFF
                    }
                }
            },
            Err(_) => {
                0xFFFFFF
            }
        };

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
                    directory: embed_directory,
                    max_age_hours: embed_max_age_hours,
                    cleanup_interval_hours: embed_cleanup_interval_hours,
                    default_color: embed_default_color,
                },
            },
            database: DatabaseConfig {
                url: database_url,
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
                    default_color: 0xFFFFFF,
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
    #[allow(dead_code)]
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
