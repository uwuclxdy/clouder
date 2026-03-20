use anyhow::Result;
use regex::Regex;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};
use tracing::debug;

const CACHE_TTL: Duration = Duration::from_secs(300);
const TRENDING_URL: &str = "https://github.com/trending";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Period {
    Daily,
    Weekly,
    Monthly,
}

impl Period {
    pub fn as_query(self) -> &'static str {
        match self {
            Period::Daily => "daily",
            Period::Weekly => "weekly",
            Period::Monthly => "monthly",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Period::Daily => "today",
            Period::Weekly => "this week",
            Period::Monthly => "this month",
        }
    }
}

#[derive(Debug, Clone)]
pub struct TrendingRepo {
    pub owner: String,
    pub name: String,
    pub description: Option<String>,
    pub language: Option<String>,
    pub stars: u64,
    pub forks: u64,
    pub stars_period: String,
}

impl TrendingRepo {
    pub fn full_name(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }

    pub fn url(&self) -> String {
        format!("https://github.com/{}/{}", self.owner, self.name)
    }
}

// --- regex patterns ---

static RE_ARTICLE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?s)<article[^>]*class="[^"]*Box-row[^"]*"[^>]*>(.*?)</article>"#).unwrap()
});

static RE_PATH: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?s)<h2[^>]*>.*?<a[^>]*href="(/[^"]+)""#).unwrap());

static RE_DESC: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?s)<p[^>]*class="[^"]*col-9[^"]*"[^>]*>(.*?)</p>"#).unwrap());

static RE_LANG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"itemprop="programmingLanguage"[^>]*>(.*?)<"#).unwrap());

static RE_STARS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?s)href="[^"]*/stargazers"[^>]*>(.*?)</a>"#).unwrap());

static RE_FORKS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?s)href="[^"]*/forks"[^>]*>(.*?)</a>"#).unwrap());

static RE_STARS_PERIOD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?s)class="[^"]*float-sm-right[^"]*"[^>]*>(.*?)</span>"#).unwrap()
});

static RE_HTML_TAGS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[^>]+>").unwrap());

// --- parsing helpers ---

fn strip_tags(html: &str) -> String {
    RE_HTML_TAGS
        .replace_all(html, "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_count(text: &str) -> u64 {
    text.replace(',', "").trim().parse().unwrap_or(0)
}

fn parse_repo(article_html: &str) -> Option<TrendingRepo> {
    let path = RE_PATH.captures(article_html)?.get(1)?.as_str();
    let (owner, name) = path.trim_start_matches('/').split_once('/')?;

    let description = RE_DESC
        .captures(article_html)
        .and_then(|c| c.get(1))
        .map(|m| strip_tags(m.as_str()))
        .filter(|s| !s.is_empty());

    let language = RE_LANG
        .captures(article_html)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string())
        .filter(|s| !s.is_empty());

    let stars = RE_STARS
        .captures(article_html)
        .and_then(|c| c.get(1))
        .map(|m| parse_count(&strip_tags(m.as_str())))
        .unwrap_or(0);

    let forks = RE_FORKS
        .captures(article_html)
        .and_then(|c| c.get(1))
        .map(|m| parse_count(&strip_tags(m.as_str())))
        .unwrap_or(0);

    let stars_period = RE_STARS_PERIOD
        .captures(article_html)
        .and_then(|c| c.get(1))
        .map(|m| strip_tags(m.as_str()))
        .unwrap_or_default();

    Some(TrendingRepo {
        owner: owner.to_string(),
        name: name.to_string(),
        description,
        language,
        stars,
        forks,
        stars_period,
    })
}

fn parse_repos(html: &str) -> Vec<TrendingRepo> {
    RE_ARTICLE
        .captures_iter(html)
        .filter_map(|cap| parse_repo(cap.get(1)?.as_str()))
        .collect()
}

// --- caching ---

struct CacheEntry {
    repos: Vec<TrendingRepo>,
    fetched_at: Instant,
}

static DAILY_CACHE: LazyLock<Mutex<Option<CacheEntry>>> = LazyLock::new(|| Mutex::new(None));
static WEEKLY_CACHE: LazyLock<Mutex<Option<CacheEntry>>> = LazyLock::new(|| Mutex::new(None));
static MONTHLY_CACHE: LazyLock<Mutex<Option<CacheEntry>>> = LazyLock::new(|| Mutex::new(None));

fn cache_for(period: Period) -> &'static Mutex<Option<CacheEntry>> {
    match period {
        Period::Daily => &DAILY_CACHE,
        Period::Weekly => &WEEKLY_CACHE,
        Period::Monthly => &MONTHLY_CACHE,
    }
}

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; clouder-bot)")
        .build()
        .expect("failed to build reqwest client")
});

pub async fn fetch_trending(period: Period) -> Result<Vec<TrendingRepo>> {
    let cache = cache_for(period);
    {
        let guard = cache.lock().unwrap();
        if let Some(entry) = &*guard
            && entry.fetched_at.elapsed() < CACHE_TTL
        {
            debug!("gh-trending: cache hit for {:?}", period);
            return Ok(entry.repos.clone());
        }
    }

    debug!("gh-trending: fetching {:?}", period);
    let url = format!("{}?since={}", TRENDING_URL, period.as_query());
    let html = CLIENT
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let repos = parse_repos(&html);

    {
        let mut guard = cache.lock().unwrap();
        *guard = Some(CacheEntry {
            repos: repos.clone(),
            fetched_at: Instant::now(),
        });
    }

    Ok(repos)
}
