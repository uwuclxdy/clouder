use anyhow::Result;
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use serenity::all::Http;
use sqlx::SqlitePool;
use std::env;
use std::sync::Arc;
use tracing::{error, info, warn};

// default color for embeds when none is configured; exposed publicly so tests and
// web handlers can reference it instead of sprinkling the magic hex value.
pub const DEFAULT_EMBED_COLOR: u32 = 0xFFFFFF; // white

const DEFAULT_API_BASE: &str = "http://127.0.0.1:8080";
const DEFAULT_BIND_ADDR: &str = "127.0.0.1:3000";
const DEFAULT_DATABASE_URL: &str = "data/db.sqlite";
const DEFAULT_OLLAMA_BASE_URL: &str = "http://localhost:11434/v1";
const DEFAULT_OPENAI_BASE_URL: &str = "https://api.openai.com/v1";
const DEFAULT_OLLAMA_MODEL: &str = "llama3.2";
const DEFAULT_OPENAI_MODEL: &str = "gpt-3.5-turbo";
const DEFAULT_LLM_TEMPERATURE: f32 = 0.7;
const DEFAULT_LLM_MAX_TOKENS: u32 = 1000;
const DEFAULT_LLM_TIMEOUT_SECONDS: u64 = 30;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub discord: DiscordConfig,
    pub web: WebConfig,
    pub database: DatabaseConfig,
    pub llm: LlmConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordConfig {
    pub token: String,
    pub application_id: u64,
    pub bot_owner: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfig {
    pub api_base: String,
    pub bind_addr: String,
    pub oauth: OAuthConfig,
    pub embed: EmbedConfig,
    pub session_secret: String,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LlmProvider {
    OpenAI,
    Ollama,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: Option<LlmProvider>,
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

fn require_env(key: &str) -> Result<String, anyhow::Error> {
    env::var(key).map_err(|e| {
        error!("{} is not set", key);
        anyhow::anyhow!("{}: {}", key, e)
    })
}

fn optional_env(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| {
        info!("{} not set, using {}", key, default);
        default.to_string()
    })
}

impl Config {
    pub fn from_env() -> Result<Self, anyhow::Error> {
        if let Err(e) = dotenv() {
            warn!("could not load .env file: {}", e);
        }

        let discord_token = require_env("DISCORD_TOKEN")?;
        // application ID == client ID for discord bots
        let oauth_client_id = require_env("DISCORD_CLIENT_ID")?;
        let application_id = oauth_client_id.parse::<u64>().map_err(|e| {
            error!(
                "invalid DISCORD_CLIENT_ID format '{}': {}",
                oauth_client_id, e
            );
            anyhow::anyhow!("invalid DISCORD_CLIENT_ID format")
        })?;
        let oauth_client_secret = require_env("DISCORD_CLIENT_SECRET")?;
        let bot_owner_str = require_env("BOT_OWNER")?;
        let bot_owner = bot_owner_str.parse::<u64>().map_err(|e| {
            error!("invalid BOT_OWNER format '{}': {}", bot_owner_str, e);
            anyhow::anyhow!("invalid BOT_OWNER format")
        })?;

        let api_base = optional_env("API_BASE", DEFAULT_API_BASE);
        let bind_addr = optional_env("WEB_BIND_ADDR", DEFAULT_BIND_ADDR);
        let database_url = optional_env("DATABASE_URL", DEFAULT_DATABASE_URL);

        let embed_default_color = match env::var("EMBED_DEFAULT_COLOR") {
            Ok(color_str) => {
                let color_str = color_str.trim();
                let parsed_color = if let Some(hex) = color_str.strip_prefix('#') {
                    u32::from_str_radix(hex, 16)
                } else if let Some(hex) = color_str
                    .strip_prefix("0x")
                    .or_else(|| color_str.strip_prefix("0X"))
                {
                    u32::from_str_radix(hex, 16)
                } else {
                    color_str.parse::<u32>()
                };

                match parsed_color {
                    Ok(color) => {
                        info!("EMBED_DEFAULT_COLOR: {:#06X}", color);
                        color
                    }
                    Err(_) => DEFAULT_EMBED_COLOR,
                }
            }
            Err(_) => DEFAULT_EMBED_COLOR,
        };

        let redirect_uri = env::var("DISCORD_REDIRECT_URI")
            .unwrap_or_else(|_| format!("{}/auth/callback", api_base));

        let session_secret = env::var("SESSION_SECRET").unwrap_or_else(|_| {
            warn!("SESSION_SECRET not set, falling back to client_secret");
            oauth_client_secret.clone()
        });

        // Parse LLM configuration
        let llm_provider = match env::var("LLM_PROVIDER") {
            Err(_) => {
                warn!("LLM_PROVIDER not set, LLM integration disabled");
                None
            }
            Ok(v) => match v.to_lowercase().as_str() {
                "openai" => Some(LlmProvider::OpenAI),
                "ollama" => Some(LlmProvider::Ollama),
                other => {
                    warn!("unknown LLM_PROVIDER '{}', LLM integration disabled", other);
                    None
                }
            },
        };

        let llm_base_url = env::var("LLM_BASE_URL").unwrap_or_else(|_| match llm_provider {
            Some(LlmProvider::Ollama) => DEFAULT_OLLAMA_BASE_URL.to_string(),
            _ => DEFAULT_OPENAI_BASE_URL.to_string(),
        });

        let llm_api_key = env::var("LLM_API_KEY").unwrap_or_else(|_| {
            if matches!(llm_provider, Some(LlmProvider::OpenAI)) {
                warn!("LLM_API_KEY not set for openai provider");
            }
            String::new()
        });

        let llm_model = env::var("LLM_MODEL").unwrap_or_else(|_| match llm_provider {
            Some(LlmProvider::Ollama) => DEFAULT_OLLAMA_MODEL.to_string(),
            _ => DEFAULT_OPENAI_MODEL.to_string(),
        });

        let llm_temperature = env::var("LLM_TEMPERATURE")
            .ok()
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(DEFAULT_LLM_TEMPERATURE);

        let llm_max_tokens = env::var("LLM_MAX_TOKENS")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(DEFAULT_LLM_MAX_TOKENS);

        let llm_timeout_seconds = env::var("LLM_TIMEOUT_SECONDS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(DEFAULT_LLM_TIMEOUT_SECONDS);

        let llm_system_prompt = env::var("LLM_SYSTEM_PROMPT").unwrap_or_default();
        let llm_stop = env::var("LLM_STOP").unwrap_or_default();

        let parse_user_ids = |env_var: &str| -> Vec<u64> {
            env::var(env_var)
                .unwrap_or_default()
                .split(',')
                .filter_map(|s| s.trim().parse::<u64>().ok())
                .collect()
        };

        let llm_allowed_users = parse_user_ids("LLM_ALLOWED_USERS");
        let llm_dm_allowed_users = parse_user_ids("LLM_DM_ALLOWED_USERS");
        let llm_no_cooldown_users = parse_user_ids("LLM_NO_COOLDOWN_USERS");

        Ok(Config {
            discord: DiscordConfig {
                token: discord_token,
                application_id,
                bot_owner,
            },
            web: WebConfig {
                api_base,
                bind_addr,
                oauth: OAuthConfig {
                    client_id: oauth_client_id,
                    client_secret: oauth_client_secret,
                    redirect_uri,
                },
                embed: EmbedConfig {
                    default_color: embed_default_color,
                },
                session_secret,
            },
            database: DatabaseConfig { url: database_url },
            llm: LlmConfig {
                provider: llm_provider,
                base_url: llm_base_url,
                api_key: llm_api_key,
                model: llm_model,
                temperature: llm_temperature,
                max_tokens: llm_max_tokens,
                timeout_seconds: llm_timeout_seconds,
                system_prompt: llm_system_prompt,
                stop: llm_stop,
                allowed_users: llm_allowed_users,
                dm_allowed_users: llm_dm_allowed_users,
                no_cooldown_users: llm_no_cooldown_users,
            },
        })
    }

    pub fn test_config() -> Self {
        Self {
            discord: DiscordConfig {
                token: "test_token".to_string(),
                application_id: 12345,
                bot_owner: 12345,
            },
            web: WebConfig {
                api_base: DEFAULT_API_BASE.to_string(),
                bind_addr: "127.0.0.1:8080".to_string(),
                oauth: OAuthConfig {
                    client_id: "12345".to_string(),
                    client_secret: "test_client_secret".to_string(),
                    redirect_uri: format!("{}/auth/callback", DEFAULT_API_BASE),
                },
                embed: EmbedConfig {
                    default_color: DEFAULT_EMBED_COLOR,
                },
                session_secret: "test_session_secret_at_least_32_bytes".to_string(),
            },
            database: DatabaseConfig {
                url: ":memory:".to_string(),
            },
            llm: LlmConfig {
                provider: None,
                base_url: DEFAULT_OPENAI_BASE_URL.to_string(),
                api_key: String::new(),
                model: DEFAULT_OPENAI_MODEL.to_string(),
                temperature: DEFAULT_LLM_TEMPERATURE,
                max_tokens: DEFAULT_LLM_MAX_TOKENS,
                timeout_seconds: DEFAULT_LLM_TIMEOUT_SECONDS,
                system_prompt: String::new(),
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
    pub http: Arc<Http>,
    #[cfg(feature = "llm")]
    pub llm_client: Option<clouder_llm::LlmClient>,
}

impl AppState {
    pub fn new(config: Arc<Config>, db: Arc<SqlitePool>, http: Arc<Http>) -> Self {
        #[cfg(feature = "llm")]
        let llm_client = config.llm.provider.as_ref().map(|_| {
            clouder_llm::LlmClient::new(
                config.llm.base_url.clone(),
                config.llm.api_key.clone(),
                config.llm.timeout_seconds,
            )
        });

        Self {
            config,
            db,
            http,
            #[cfg(feature = "llm")]
            llm_client,
        }
    }
}
