use anyhow::Result;
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use tracing::debug;

const CACHE_TTL: Duration = Duration::from_secs(300);
pub const FETCH_LIMIT: usize = 10;

// Handles fields that may be absent OR explicitly null in the API response.
fn null_as_zero<'de, D: Deserializer<'de>>(d: D) -> Result<u64, D::Error> {
    Option::<u64>::deserialize(d).map(|v| v.unwrap_or(0))
}

#[derive(Debug, Clone, Deserialize)]
pub struct HfModel {
    pub id: String,
    pub author: Option<String>,
    #[serde(default, deserialize_with = "null_as_zero")]
    pub downloads: u64,
    #[serde(default, deserialize_with = "null_as_zero")]
    pub likes: u64,
    #[serde(rename = "pipeline_tag")]
    pub pipeline_tag: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(rename = "lastModified")]
    pub last_modified: Option<String>,
    #[serde(rename = "cardData")]
    pub card_data: Option<Value>,
}

impl HfModel {
    /// Extracts the short model name from the `author/model` id format.
    pub fn short_name(&self) -> &str {
        self.id.split('/').next_back().unwrap_or(&self.id)
    }

    /// Returns the resolved author: explicit field or the prefix before `/` in id.
    pub fn resolved_author(&self) -> Option<&str> {
        self.author.as_deref().or_else(|| {
            self.id
                .contains('/')
                .then(|| self.id.split('/').next())
                .flatten()
        })
    }

    /// Returns the description from the model card, if present.
    pub fn description(&self) -> Option<&str> {
        self.card_data.as_ref()?.get("description")?.as_str()
    }

    /// Returns up to `limit` tags that are neither framework labels nor license strings.
    pub fn relevant_tags(&self, limit: usize) -> Vec<&str> {
        let skip = [
            "transformers",
            "pytorch",
            "jax",
            "tf",
            "safetensors",
            "gguf",
        ];
        self.tags
            .iter()
            .filter(|t| {
                !t.starts_with("license:")
                    && !t.starts_with("region:")
                    && !skip.contains(&t.as_str())
            })
            .map(String::as_str)
            .take(limit)
            .collect()
    }
}

struct ModelCache {
    models: Vec<HfModel>,
    fetched_at: Instant,
}

static TRENDING_CACHE: OnceLock<Mutex<Option<ModelCache>>> = OnceLock::new();
static LATEST_CACHE: OnceLock<Mutex<Option<ModelCache>>> = OnceLock::new();

pub async fn fetch_trending() -> Result<Vec<HfModel>> {
    let url = format!(
        "https://huggingface.co/api/models?sort=downloads&direction=-1&limit={}",
        FETCH_LIMIT
    );
    fetch_or_refresh(TRENDING_CACHE.get_or_init(|| Mutex::new(None)), &url).await
}

pub async fn fetch_latest() -> Result<Vec<HfModel>> {
    let url = format!(
        "https://huggingface.co/api/models?sort=lastModified&direction=-1&limit={}",
        FETCH_LIMIT
    );
    fetch_or_refresh(LATEST_CACHE.get_or_init(|| Mutex::new(None)), &url).await
}

async fn fetch_or_refresh(
    cache: &'static Mutex<Option<ModelCache>>,
    url: &str,
) -> Result<Vec<HfModel>> {
    let cached = {
        let guard = cache.lock().unwrap();
        guard.as_ref().and_then(|c| {
            if c.fetched_at.elapsed() < CACHE_TTL {
                Some(c.models.clone())
            } else {
                None
            }
        })
    };

    if let Some(models) = cached {
        debug!("hf: cache hit");
        return Ok(models);
    }

    debug!("hf: fetching from api");
    let models: Vec<HfModel> = reqwest::get(url).await?.json().await?;

    {
        let mut guard = cache.lock().unwrap();
        *guard = Some(ModelCache {
            models: models.clone(),
            fetched_at: Instant::now(),
        });
    }

    Ok(models)
}
