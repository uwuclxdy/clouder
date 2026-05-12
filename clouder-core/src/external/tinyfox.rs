use anyhow::Result;
use serde::Deserialize;
use std::sync::OnceLock;
use tracing::debug;
use url::Url;

const BASE_URL: &str = "https://api.tinyfox.dev";
const IMAGE_HOST: &str = "https://tinyfox.dev";

#[derive(Deserialize)]
struct ImgResponse {
    loc: String,
}

static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

fn client() -> &'static reqwest::Client {
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .user_agent("clouder-bot")
            .build()
            .expect("failed to build reqwest client")
    })
}

pub async fn fetch_animal_image(animal: &str) -> Result<String> {
    debug!("fetching tinyfox animal: {}", animal);
    let url = format!("{BASE_URL}/img.json?animal={animal}");
    let resp: ImgResponse = client()
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(format!("{IMAGE_HOST}{}", resp.loc))
}

pub fn progress_url(period: &str, tz: Option<&str>) -> String {
    let mut url = Url::parse(&format!("{BASE_URL}/progress/{period}")).unwrap();
    if let Some(tz) = tz {
        url.query_pairs_mut().append_pair("tz", tz);
    }
    url.to_string()
}
