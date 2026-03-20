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
        self.pushed_at.as_deref().map(|s| s.get(..10).unwrap_or(s))
    }
}

type ReposMap = HashMap<String, (Vec<GhRepo>, Instant)>;

static USER_CACHE: OnceLock<Mutex<HashMap<String, (GhUser, Instant)>>> = OnceLock::new();
static REPO_CACHE: OnceLock<Mutex<HashMap<String, (GhRepo, Instant)>>> = OnceLock::new();
static REPOS_CACHE: OnceLock<Mutex<ReposMap>> = OnceLock::new();

fn user_cache() -> &'static Mutex<HashMap<String, (GhUser, Instant)>> {
    USER_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn repo_cache() -> &'static Mutex<HashMap<String, (GhRepo, Instant)>> {
    REPO_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn repos_cache() -> &'static Mutex<ReposMap> {
    REPOS_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

fn client() -> &'static reqwest::Client {
    CLIENT.get_or_init(|| {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::USER_AGENT,
            reqwest::header::HeaderValue::from_static("clouder-bot"),
        );
        headers.insert(
            reqwest::header::ACCEPT,
            reqwest::header::HeaderValue::from_static("application/vnd.github+json"),
        );
        reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("failed to build reqwest client")
    })
}

pub async fn fetch_user(username: &str, token: Option<&str>) -> Result<GhUser> {
    {
        let cache = user_cache().lock().unwrap();
        if let Some((user, at)) = cache.get(username)
            && at.elapsed() < CACHE_TTL
        {
            debug!("user cache hit for {}", username);
            return Ok(user.clone());
        }
    }

    debug!("fetching user {}", username);
    let url = format!("{GITHUB_API}/users/{username}");
    let mut req = client().get(&url);
    if let Some(t) = token {
        req = req.bearer_auth(t);
    }
    let resp = req.send().await?;

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
            debug!("repo cache hit for {}", key);
            return Ok(r.clone());
        }
    }

    debug!("fetching repo {}", key);
    let url = format!("{GITHUB_API}/repos/{key}");
    let mut req = client().get(&url);
    if let Some(t) = token {
        req = req.bearer_auth(t);
    }
    let resp = req.send().await?;

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

pub async fn fetch_repos(username: &str, token: Option<&str>) -> Result<Vec<GhRepo>> {
    {
        let cache = repos_cache().lock().unwrap();
        if let Some((repos, at)) = cache.get(username)
            && at.elapsed() < CACHE_TTL
        {
            debug!("repos cache hit for {}", username);
            return Ok(repos.clone());
        }
    }

    debug!("fetching repos for {}", username);
    let url = format!("{GITHUB_API}/users/{username}/repos?per_page=100");
    let mut req = client().get(&url);
    if let Some(t) = token {
        req = req.bearer_auth(t);
    }
    let resp = req.send().await?;

    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        bail!("not found");
    }
    let mut repos: Vec<GhRepo> = resp.error_for_status()?.json().await?;
    repos.sort_by(|a, b| b.stargazers_count.cmp(&a.stargazers_count));

    {
        let mut cache = repos_cache().lock().unwrap();
        cache.insert(username.to_string(), (repos.clone(), Instant::now()));
    }

    Ok(repos)
}
