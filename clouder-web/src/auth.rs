use crate::WebState;
use crate::session;
use axum::extract::{Query, State};
use axum::response::Redirect;
use axum_extra::extract::cookie::SignedCookieJar;
use clouder_core::DashboardUser;
use clouder_core::database::dashboard_sessions::DashboardSession;
use serde::Deserialize;
use tracing::{error, info, warn};

#[derive(Deserialize)]
pub struct OAuthCallback {
    code: Option<String>,
    error: Option<String>,
}

#[derive(Debug)]
struct DiscordProfile {
    user_id: String,
    username: String,
    avatar: Option<String>,
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

    let profile = match fetch_user(&access_token).await {
        Ok(u) => u,
        Err(e) => {
            error!("discord user fetch failed: {}", e);
            return (jar, Redirect::to("/login?error=auth_failed"));
        }
    };

    info!("user {} ({}) logged in", profile.username, profile.user_id);

    if let Err(e) = DashboardUser::upsert(
        &state.app_state.db,
        &profile.user_id,
        &state.app_state.config.web.api_key_pepper,
    )
    .await
    {
        error!("failed to upsert dashboard user: {}", e);
        return (jar, Redirect::to("/login?error=auth_failed"));
    }
    let encrypted_token = match clouder_core::crypto::encrypt(
        &state.app_state.config.web.oauth_encryption_key_bytes,
        access_token.as_bytes(),
    ) {
        Ok(c) => c,
        Err(e) => {
            error!("failed to encrypt oauth token: {}", e);
            return (jar, Redirect::to("/login?error=auth_failed"));
        }
    };
    if let Err(e) =
        DashboardUser::store_oauth_token(&state.app_state.db, &profile.user_id, &encrypted_token)
            .await
    {
        error!("failed to store oauth token: {}", e);
        return (jar, Redirect::to("/login?error=auth_failed"));
    }
    if let Err(e) = DashboardUser::store_profile(
        &state.app_state.db,
        &profile.user_id,
        &profile.username,
        profile.avatar.as_deref(),
    )
    .await
    {
        warn!("failed to store profile: {}", e);
    }

    if let Err(e) =
        clouder_core::shared::refresh_guild_cache(&state.app_state, &profile.user_id, &access_token)
            .await
    {
        warn!("guild cache refresh on login failed: {}", e);
    }

    let session = match DashboardSession::create(
        &state.app_state.db,
        &profile.user_id,
        session::SESSION_TTL_SECONDS,
    )
    .await
    {
        Ok(s) => s,
        Err(e) => {
            error!("failed to create session: {}", e);
            return (jar, Redirect::to("/login?error=auth_failed"));
        }
    };

    let secure = state.app_state.config.web.api_base.starts_with("https://");
    let new_jar = session::store_cookie(jar, &session.session_id, secure);
    (new_jar, Redirect::to("/servers"))
}

pub async fn logout(
    State(state): State<WebState>,
    jar: SignedCookieJar,
) -> (SignedCookieJar, Redirect) {
    if let Some(session_id) = session::read_cookie(&jar) {
        let _ = DashboardSession::delete(&state.app_state.db, &session_id).await;
    }
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

async fn fetch_user(access_token: &str) -> Result<DiscordProfile, String> {
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
    Ok(DiscordProfile {
        user_id: json["id"].as_str().unwrap_or("").to_string(),
        username: json["username"].as_str().unwrap_or("unknown").to_string(),
        avatar: json["avatar"].as_str().map(|s| s.to_string()),
    })
}
