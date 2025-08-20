use axum::http::{HeaderMap, StatusCode};
use crate::web::{middleware::{GLOBAL_SESSION_STORE, SessionData}, models::SessionUser};

pub async fn extract_session_data(headers: &HeaderMap) -> Result<SessionData, StatusCode> {
    let session_id = extract_session_id_from_headers(headers)?;
    
    let session = match GLOBAL_SESSION_STORE.get_session(&session_id).await {
        Some(session) => session,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    // Check expiration
    if session.expires_at < chrono::Utc::now() {
        GLOBAL_SESSION_STORE.delete_session(&session_id).await;
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Extract user data
    let user_data = session
        .data
        .get("user")
        .and_then(|value| serde_json::from_value::<SessionUser>(value.clone()).ok());

    Ok((session, user_data))
}

pub fn extract_session_id_from_headers(headers: &HeaderMap) -> Result<String, StatusCode> {
    let cookie_header = headers
        .get(axum::http::header::COOKIE)
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    for cookie in cookie_header.split(';') {
        let cookie = cookie.trim();
        if let Some(session_id) = cookie.strip_prefix("session_id=") {
            return Ok(session_id.to_string());
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}