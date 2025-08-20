use crate::config::AppState;
use crate::web::{middleware::GLOBAL_SESSION_STORE, models::{DiscordUser, Guild, SessionUser}};
use axum::{
    extract::{Query, State},
    response::Redirect,
    http::{StatusCode, HeaderValue},
};
use serde::Deserialize;
use reqwest::Client;

#[derive(Deserialize)]
pub struct AuthQuery {
    code: Option<String>,
    #[allow(dead_code)]
    state: Option<String>,
    error: Option<String>,
}

#[derive(Deserialize)]
pub struct TokenResponse {
    access_token: String,
    #[allow(dead_code)]
    token_type: String,
    #[allow(dead_code)]
    expires_in: u64,
    #[allow(dead_code)]
    refresh_token: String,
    #[allow(dead_code)]
    scope: String,
}

pub async fn login(State(state): State<AppState>) -> Redirect {
    let auth_url = format!(
        "https://discord.com/api/oauth2/authorize?client_id={}&redirect_uri={}&response_type=code&scope=identify%20guilds",
        state.config.web.oauth.client_id,
        urlencoding::encode(&state.config.web.oauth.redirect_uri)
    );
    Redirect::temporary(&auth_url)
}

pub async fn callback(
    State(state): State<AppState>,
    Query(params): Query<AuthQuery>,
) -> Result<(StatusCode, [(axum::http::HeaderName, HeaderValue); 1], Redirect), Redirect> {
    if params.error.is_some() || params.code.is_none() {
        return Err(Redirect::temporary("/auth/login"));
    }
    
    let code = params.code.unwrap();
    let client = Client::new();
    
    // Exchange code for token
    let token_params = [
        ("client_id", state.config.web.oauth.client_id.as_str()),
        ("client_secret", state.config.web.oauth.client_secret.as_str()),
        ("grant_type", "authorization_code"),
        ("code", &code),
        ("redirect_uri", &state.config.web.oauth.redirect_uri),
    ];

    let token_response = match client
        .post("https://discord.com/api/oauth2/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&token_params)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => resp,
        _ => return Err(Redirect::temporary("/auth/login")),
    };

    let token_data: TokenResponse = match token_response.json().await {
        Ok(data) => data,
        Err(_) => return Err(Redirect::temporary("/auth/login")),
    };

    // Get user info
    let user: DiscordUser = match client
        .get("https://discord.com/api/users/@me")
        .bearer_auth(&token_data.access_token)
        .send()
        .await
        .and_then(|resp| resp.error_for_status())
        .map_err(|_| ())
    {
        Ok(resp) => match resp.json().await {
            Ok(user) => user,
            Err(_) => return Err(Redirect::temporary("/auth/login")),
        },
        Err(_) => return Err(Redirect::temporary("/auth/login")),
    };

    // Get user guilds
    let guilds_response = client
        .get("https://discord.com/api/users/@me/guilds")
        .bearer_auth(&token_data.access_token)
        .send()
        .await;

    let guilds: Vec<Guild> = match guilds_response {
        Ok(resp) => {
            let guilds_text = resp.text().await.unwrap_or_default();
            serde_json::from_str(&guilds_text).unwrap_or_else(|e| {
                tracing::error!("Failed to parse guilds: {}", e);
                Vec::new()
            })
        }
        Err(_) => Vec::new(),
    };

    let session_user = SessionUser { user, guilds, access_token: token_data.access_token };

    // Create session
    let session_id = GLOBAL_SESSION_STORE.create_session().await;
    
    if let Some(mut session) = GLOBAL_SESSION_STORE.get_session(&session_id).await {
        session.data.insert("user".to_string(), serde_json::to_value(&session_user).unwrap());
        GLOBAL_SESSION_STORE.update_session(&session_id, session).await;
    } else {
        return Err(Redirect::temporary("/auth/login"));
    }

    let cookie_header = HeaderValue::from_str(&format!("session_id={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=86400", session_id)).unwrap();

    Ok((
        StatusCode::FOUND,
        [(axum::http::header::SET_COOKIE, cookie_header)],
        Redirect::temporary("/")
    ))
}

pub async fn logout() -> (StatusCode, [(axum::http::HeaderName, HeaderValue); 1], Redirect) {
    let cookie_header = HeaderValue::from_str("session_id=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0").unwrap();
    (
        StatusCode::FOUND,
        [(axum::http::header::SET_COOKIE, cookie_header)],
        Redirect::temporary("/auth/login")
    )
}
