use crate::WebState;
use crate::session::{self, SessionUser};
use axum::extract::{Query, State};
use axum::response::Redirect;
use axum_extra::extract::cookie::SignedCookieJar;
use serde::Deserialize;
use tracing::{error, info, warn};

#[derive(Deserialize)]
pub struct OAuthCallback {
    code: Option<String>,
    error: Option<String>,
}

pub async fn login(State(state): State<WebState>) -> Redirect {
    let oauth = &state.app_state.config.web.oauth;
    let url = format!(
        "https://discord.com/oauth2/authorize?client_id={}&response_type=code&redirect_uri={}&scope=identify+guilds",
        oauth.client_id,
        urlencoding::encode(&oauth.redirect_uri),
    );
    Redirect::temporary(&url)
}

pub async fn callback(
    State(state): State<WebState>,
    Query(query): Query<OAuthCallback>,
    jar: SignedCookieJar,
) -> (SignedCookieJar, Redirect) {
    if let Some(err) = query.error {
        warn!("discord oauth error: {}", err);
        return (jar, Redirect::to("/login?error=denied"));
    }

    let code = match query.code {
        Some(c) => c,
        None => return (jar, Redirect::to("/login?error=missing_code")),
    };

    let access_token = match exchange_code(&state, &code).await {
        Ok(t) => t,
        Err(e) => {
            error!("token exchange failed: {}", e);
            return (jar, Redirect::to("/login?error=auth_failed"));
        }
    };

    let user = match fetch_user(&access_token).await {
        Ok(u) => u,
        Err(e) => {
            error!("discord user fetch failed: {}", e);
            return (jar, Redirect::to("/login?error=auth_failed"));
        }
    };

    info!("user {} ({}) logged in", user.username, user.user_id);
    let secure = state.app_state.config.web.api_url.starts_with("https://");
    (session::store(jar, &user, secure), Redirect::to("/servers"))
}

pub async fn logout(jar: SignedCookieJar) -> (SignedCookieJar, Redirect) {
    (session::clear(jar), Redirect::to("/"))
}

async fn exchange_code(state: &WebState, code: &str) -> Result<String, String> {
    let oauth = &state.app_state.config.web.oauth;
    let client = reqwest::Client::new();

    let response = client
        .post("https://discord.com/api/oauth2/token")
        .form(&[
            ("client_id", oauth.client_id.as_str()),
            ("client_secret", oauth.client_secret.as_str()),
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", oauth.redirect_uri.as_str()),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(format!("discord token endpoint returned error: {}", body));
    }

    let json: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    json["access_token"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "missing access_token in discord response".to_string())
}

async fn fetch_user(access_token: &str) -> Result<SessionUser, String> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://discord.com/api/users/@me")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(format!("discord user endpoint returned error: {}", body));
    }
    let json: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    Ok(SessionUser {
        user_id: json["id"].as_str().unwrap_or("").to_string(),
        username: json["username"].as_str().unwrap_or("unknown").to_string(),
        avatar: json["avatar"].as_str().map(|s| s.to_string()),
        access_token: access_token.to_string(),
    })
}
