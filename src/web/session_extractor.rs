use crate::web::{
    middleware::{SessionData, GLOBAL_SESSION_STORE},
    models::SessionUser,
};
use axum::http::{HeaderMap, StatusCode};

pub async fn extract_session_data(headers: &HeaderMap) -> Result<SessionData, StatusCode> {
    let session_id = extract_session_id_from_headers(headers)?;

    let session = GLOBAL_SESSION_STORE
        .get_session(&session_id)
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if session.expires_at < chrono::Utc::now() {
        GLOBAL_SESSION_STORE.delete_session(&session_id).await;
        return Err(StatusCode::UNAUTHORIZED);
    }

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
