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
    pub openai: OpenAIConfig,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIConfig {
    pub enabled: bool,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub timeout_seconds: u64,
    pub system_prompt: String,
    pub stop: String,
    pub allowed_users: Vec<u64>,
    pub dm_allowed_users: Vec<u64>,
    pub no_cooldown_users: Vec<u64>,
}

impl Config {
    pub fn from_env() -> Result<Self, anyhow::Error> {
        if let Err(e) = dotenv() {
            warn!(
                "Could not load .env file: {}. Continuing with system environment variables.",
                e
            );
        }

        let discord_token = match env::var("DISCORD_TOKEN") {
            Ok(token) => token,
            Err(_) => {
                error!(
                    "DISCORD_TOKEN environment variable not set - this is required for the bot to function"
                );
                return Err(anyhow::anyhow!(
                    "DISCORD_TOKEN environment variable not set"
                ));
            }
        };
        let application_id = match env::var("DISCORD_APPLICATION_ID") {
            Ok(id_str) => match id_str.parse::<u64>() {
                Ok(id) => id,
                Err(e) => {
                    error!(
                        "DISCORD_APPLICATION_ID has invalid format '{}': {}",
                        id_str, e
                    );
                    return Err(anyhow::anyhow!("Invalid DISCORD_APPLICATION_ID format"));
                }
            },
            Err(_) => {
                error!(
                    "DISCORD_APPLICATION_ID environment variable not set - this is required for Discord interactions"
                );
                return Err(anyhow::anyhow!(
                    "DISCORD_APPLICATION_ID environment variable not set"
                ));
            }
        };
        let oauth_client_id = match env::var("DISCORD_CLIENT_ID") {
            Ok(id) => id,
            Err(_) => {
                error!(
                    "DISCORD_CLIENT_ID environment variable not set - this is required for OAuth"
                );
                return Err(anyhow::anyhow!(
                    "DISCORD_CLIENT_ID environment variable not set"
                ));
            }
        };

        let oauth_client_secret = match env::var("DISCORD_CLIENT_SECRET") {
            Ok(secret) => secret,
            Err(_) => {
                error!(
                    "DISCORD_CLIENT_SECRET environment variable not set - this is required for OAuth"
                );
                return Err(anyhow::anyhow!(
                    "DISCORD_CLIENT_SECRET environment variable not set"
                ));
            }
        };
        let base_url = match env::var("BASE_URL") {
            Ok(url) => {
                info!("Using custom BASE_URL: {}", url);
                url
            }
            Err(_) => {
                info!("BASE_URL not set, using default: http://localhost:3000");
                "http://localhost:3000".to_string()
            }
        };

        let host = match env::var("HOST") {
            Ok(host) => {
                info!("Using custom HOST: {}", host);
                host
            }
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
                }
                Err(e) => {
                    warn!(
                        "PORT '{}' has invalid format: {}. Using default port 3000",
                        port_str, e
                    );
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
            }
            Err(_) => {
                info!("DATABASE_URL not set, using default: data/db.sqlite");
                "data/db.sqlite".to_string()
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
                    }
                    Err(_) => 0xFFFFFF,
                }
            }
            Err(_) => 0xFFFFFF,
        };

        let redirect_uri = format!("{}/auth/callback", base_url);

        // Parse OpenAI configuration
        let openai_enabled = env::var("OPENAI_ENABLED")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);

        let openai_base_url =
            env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".to_string());

        let openai_api_key = env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
            warn!("OPENAI_API_KEY not set - OpenAI integration will not work without an API key");
            String::new()
        });

        let openai_model = env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-3.5-turbo".to_string());

        let openai_temperature = env::var("OPENAI_TEMPERATURE")
            .unwrap_or_else(|_| "0.7".to_string())
            .parse::<f32>()
            .unwrap_or(0.7);

        let openai_max_tokens = env::var("OPENAI_MAX_TOKENS")
            .unwrap_or_else(|_| "1000".to_string())
            .parse::<u32>()
            .unwrap_or(1000);

        let openai_timeout_seconds = env::var("OPENAI_TIMEOUT_SECONDS")
            .unwrap_or_else(|_| "30".to_string())
            .parse::<u64>()
            .unwrap_or(30);

        let openai_system_prompt = env::var("OPENAI_SYSTEM_PROMPT").unwrap_or_else(|_| {
            "You are a helpful Discord bot assistant. Respond concisely and friendly.".to_string()
        });

        let openai_stop = env::var("OPENAI_STOP").unwrap_or_else(|_| String::new());

        let parse_user_ids = |env_var: &str| -> Vec<u64> {
            env::var(env_var)
                .unwrap_or_default()
                .split(',')
                .filter_map(|s| s.trim().parse::<u64>().ok())
                .collect()
        };

        let openai_allowed_users = parse_user_ids("OPENAI_ALLOWED_USERS");
        let openai_dm_allowed_users = parse_user_ids("OPENAI_DM_ALLOWED_USERS");
        let openai_no_cooldown_users = parse_user_ids("OPENAI_NO_COOLDOWN_USERS");

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
                    default_color: embed_default_color,
                },
            },
            database: DatabaseConfig { url: database_url },
            openai: OpenAIConfig {
                enabled: openai_enabled,
                base_url: openai_base_url,
                api_key: openai_api_key,
                model: openai_model,
                temperature: openai_temperature,
                max_tokens: openai_max_tokens,
                timeout_seconds: openai_timeout_seconds,
                system_prompt: openai_system_prompt,
                stop: openai_stop,
                allowed_users: openai_allowed_users,
                dm_allowed_users: openai_dm_allowed_users,
                no_cooldown_users: openai_no_cooldown_users,
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
                    default_color: 0xFFFFFF,
                },
            },
            database: DatabaseConfig {
                url: ":memory:".to_string(),
            },
            openai: OpenAIConfig {
                enabled: false,
                base_url: "https://api.openai.com/v1".to_string(),
                api_key: "test-api-key".to_string(),
                model: "gpt-3.5-turbo".to_string(),
                temperature: 0.7,
                max_tokens: 1000,
                timeout_seconds: 30,
                system_prompt:
                    "You are a helpful Discord bot assistant. Respond concisely and friendly."
                        .to_string(),
                stop: String::new(),
                allowed_users: vec![],
                dm_allowed_users: vec![],
                no_cooldown_users: vec![],
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
    pub openai_client: Option<crate::external::openai::OpenAIClient>,
}

impl AppState {
    pub fn new(
        config: Arc<Config>,
        db: Arc<SqlitePool>,
        cache: Arc<Cache>,
        http: Arc<Http>,
    ) -> Self {
        let openai_client = if config.openai.enabled {
            Some(crate::external::openai::OpenAIClient::new(
                config.openai.base_url.clone(),
                config.openai.api_key.clone(),
                config.openai.timeout_seconds,
            ))
        } else {
            None
        };

        Self {
            config,
            db,
            cache,
            http,
            openai_client,
        }
    }
}
