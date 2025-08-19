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
    state: Option<String>,
    error: Option<String>,
}

#[derive(Deserialize)]
pub struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
    refresh_token: String,
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
    // Handle OAuth2 errors
    if params.error.is_some() {
        return Err(Redirect::temporary("/auth/login"));
    }
    
    let code = match params.code {
        Some(code) => code,
        None => return Err(Redirect::temporary("/auth/login")),
    };

    let client = Client::new();
    
    // Exchange authorization code for access token
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
        Ok(resp) => resp,
        Err(e) => {
            tracing::error!("Failed to exchange code for token: {}", e);
            return Err(Redirect::temporary("/auth/login"));
        }
    };

    if !token_response.status().is_success() {
        tracing::error!("Discord OAuth2 token exchange failed: {}", token_response.status());
        return Err(Redirect::temporary("/auth/login"));
    }

    let token_data: TokenResponse = match token_response.json().await {
        Ok(data) => data,
        Err(e) => {
            tracing::error!("Failed to parse token response: {}", e);
            return Err(Redirect::temporary("/auth/login"));
        }
    };

    // Get user information
    let user_response = match client
        .get("https://discord.com/api/users/@me")
        .bearer_auth(&token_data.access_token)
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            tracing::error!("Failed to fetch user info: {}", e);
            return Err(Redirect::temporary("/auth/login"));
        }
    };

    let user: DiscordUser = match user_response.json().await {
        Ok(user) => user,
        Err(e) => {
            tracing::error!("Failed to parse user response: {}", e);
            return Err(Redirect::temporary("/auth/login"));
        }
    };

    // Get user guilds
    let guilds_response = match client
        .get("https://discord.com/api/users/@me/guilds")
        .bearer_auth(&token_data.access_token)
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            tracing::error!("Failed to fetch user guilds: {}", e);
            return Err(Redirect::temporary("/auth/login"));
        }
    };

    // Get the raw response text for debugging
    let guilds_text = match guilds_response.text().await {
        Ok(text) => text,
        Err(e) => {
            tracing::error!("Failed to get guilds response text: {}", e);
            return Err(Redirect::temporary("/auth/login"));
        }
    };
    
    tracing::debug!("Raw guilds response: {}", guilds_text);
    
    let guilds: Vec<Guild> = match serde_json::from_str::<Vec<Guild>>(&guilds_text) {
        Ok(guilds) => {
            tracing::info!("Successfully parsed {} guilds", guilds.len());
            guilds
        }
        Err(e) => {
            tracing::error!("Failed to parse guilds response: {}", e);
            tracing::error!("Raw response was: {}", guilds_text);
            
            // Try to parse as a more generic JSON value to see the structure
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&guilds_text) {
                tracing::error!("JSON structure: {:#}", json_value);
            }
            
            // For now, continue with empty guilds to at least allow login
            tracing::warn!("Continuing with empty guilds list due to parsing error");
            Vec::new()
        }
    };

    // Create session user
    let session_user = SessionUser {
        user,
        guilds,
        access_token: token_data.access_token,
    };

    // Create new session
    let session_id = GLOBAL_SESSION_STORE.create_session().await;
    tracing::info!("Created new session: {}", session_id);
    
    // Store user data in session
    if let Some(mut session) = GLOBAL_SESSION_STORE.get_session(&session_id).await {
        session.data.insert("user".to_string(), serde_json::to_value(&session_user).unwrap());
        GLOBAL_SESSION_STORE.update_session(&session_id, session).await;
        tracing::info!("Stored user data in session for: {}", session_user.user.username);
    } else {
        tracing::error!("Failed to get session after creation: {}", session_id);
        return Err(Redirect::temporary("/auth/login"));
    }

    // Set session cookie and redirect
    let cookie_header = HeaderValue::from_str(&format!("session_id={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=86400", session_id))
        .unwrap();
    
    tracing::info!("Setting session cookie: session_id={}", session_id);

    Ok((
        StatusCode::FOUND,
        [(axum::http::header::SET_COOKIE, cookie_header)],
        Redirect::temporary("/")
    ))
}

pub async fn logout() -> (StatusCode, [(axum::http::HeaderName, HeaderValue); 1], Redirect) {
    // Clear session cookie
    let cookie_header = HeaderValue::from_str("session_id=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0").unwrap();
    
    (
        StatusCode::FOUND,
        [(axum::http::header::SET_COOKIE, cookie_header)],
        Redirect::temporary("/auth/login")
    )
}
