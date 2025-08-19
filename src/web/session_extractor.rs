use axum::http::{HeaderMap, StatusCode};
use crate::web::{middleware::{GLOBAL_SESSION_STORE, SessionData}, models::SessionUser};

pub async fn extract_session_data(headers: &HeaderMap) -> Result<SessionData, StatusCode> {
    let session_id = match extract_session_id_from_headers(headers) {
        Ok(id) => id,
        Err(e) => {
            tracing::debug!("No session cookie found: {:?}", e);
            return Err(e);
        }
    };
    
    tracing::debug!("Extracting session data for session_id: {}", session_id);
    
    let session = match GLOBAL_SESSION_STORE.get_session(&session_id).await {
        Some(session) => session,
        None => {
            tracing::debug!("Session not found in store: {}", session_id);
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // Check if session is expired
    if session.expires_at < chrono::Utc::now() {
        tracing::debug!("Session expired: {}", session_id);
        GLOBAL_SESSION_STORE.delete_session(&session_id).await;
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Try to extract user data from session
    let user_data = session
        .data
        .get("user")
        .and_then(|value| {
            match serde_json::from_value::<SessionUser>(value.clone()) {
                Ok(user) => {
                    tracing::debug!("Successfully extracted user data for: {}", user.user.username);
                    Some(user)
                }
                Err(e) => {
                    tracing::error!("Failed to deserialize user data: {}", e);
                    None
                }
            }
        });

    if user_data.is_none() {
        tracing::debug!("No user data found in session");
    }

    Ok((session, user_data))
}

fn extract_session_id_from_headers(headers: &HeaderMap) -> Result<String, StatusCode> {
    let cookie_header = headers
        .get(axum::http::header::COOKIE)
        .ok_or_else(|| {
            tracing::debug!("No Cookie header found");
            StatusCode::UNAUTHORIZED
        })?
        .to_str()
        .map_err(|e| {
            tracing::error!("Invalid cookie header: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    tracing::debug!("Cookie header: {}", cookie_header);

    for cookie in cookie_header.split(';') {
        let cookie = cookie.trim();
        if let Some(session_id) = cookie.strip_prefix("session_id=") {
            tracing::debug!("Found session_id: {}", session_id);
            return Ok(session_id.to_string());
        }
    }

    tracing::debug!("No session_id cookie found in: {}", cookie_header);
    Err(StatusCode::UNAUTHORIZED)
}