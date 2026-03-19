use anyhow::{Result, bail};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use tracing::debug;

const CACHE_TTL: Duration = Duration::from_secs(300);
const GITHUB_API: &str = "https://api.github.com";

#[derive(Debug, Clone, Deserialize)]
pub struct GhUser {
    pub login: String,
    pub name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: String,
    pub html_url: String,
    pub public_repos: u32,
    pub followers: u32,
    pub following: u32,
    pub location: Option<String>,
    pub blog: Option<String>,
    pub company: Option<String>,
}

impl GhUser {
    pub fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.login)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct GhOwner {
    pub avatar_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GhLicense {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GhRepo {
    pub full_name: String,
    pub description: Option<String>,
    pub html_url: String,
    pub stargazers_count: u32,
    pub forks_count: u32,
    pub open_issues_count: u32,
    pub language: Option<String>,
    pub pushed_at: Option<String>,
    #[serde(default)]
    pub topics: Vec<String>,
    pub license: Option<GhLicense>,
    pub owner: GhOwner,
}

impl GhRepo {
    pub fn pushed_date(&self) -> Option<&str> {
        self.pushed_at.as_deref().map(|s| &s[..s.len().min(10)])
    }
}

static USER_CACHE: OnceLock<Mutex<HashMap<String, (GhUser, Instant)>>> = OnceLock::new();
static REPO_CACHE: OnceLock<Mutex<HashMap<String, (GhRepo, Instant)>>> = OnceLock::new();

fn user_cache() -> &'static Mutex<HashMap<String, (GhUser, Instant)>> {
    USER_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn repo_cache() -> &'static Mutex<HashMap<String, (GhRepo, Instant)>> {
    REPO_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn github_client(token: Option<&str>) -> Result<reqwest::Client> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static("clouder-bot"),
    );
    headers.insert(
        reqwest::header::ACCEPT,
        reqwest::header::HeaderValue::from_static("application/vnd.github+json"),
    );
    if let Some(t) = token {
        let val = reqwest::header::HeaderValue::from_str(&format!("Bearer {t}"))?;
        headers.insert(reqwest::header::AUTHORIZATION, val);
    }
    Ok(reqwest::Client::builder()
        .default_headers(headers)
        .build()?)
}

pub async fn fetch_user(username: &str, token: Option<&str>) -> Result<GhUser> {
    {
        let cache = user_cache().lock().unwrap();
        if let Some((user, at)) = cache.get(username)
            && at.elapsed() < CACHE_TTL
        {
            debug!("github: user cache hit for {}", username);
            return Ok(user.clone());
        }
    }

    debug!("github: fetching user {}", username);
    let url = format!("{GITHUB_API}/users/{username}");
    let resp = github_client(token)?.get(&url).send().await?;

    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        bail!("not found");
    }
    let user: GhUser = resp.error_for_status()?.json().await?;

    {
        let mut cache = user_cache().lock().unwrap();
        cache.insert(username.to_string(), (user.clone(), Instant::now()));
    }

    Ok(user)
}

pub async fn fetch_repo(owner: &str, repo: &str, token: Option<&str>) -> Result<GhRepo> {
    let key = format!("{owner}/{repo}");

    {
        let cache = repo_cache().lock().unwrap();
        if let Some((r, at)) = cache.get(&key)
            && at.elapsed() < CACHE_TTL
        {
            debug!("github: repo cache hit for {}", key);
            return Ok(r.clone());
        }
    }

    debug!("github: fetching repo {}", key);
    let url = format!("{GITHUB_API}/repos/{key}");
    let resp = github_client(token)?.get(&url).send().await?;

    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        bail!("not found");
    }
    let r: GhRepo = resp.error_for_status()?.json().await?;

    {
        let mut cache = repo_cache().lock().unwrap();
        cache.insert(key, (r.clone(), Instant::now()));
    }

    Ok(r)
}
