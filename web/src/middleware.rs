use crate::models::SessionUser;
use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub data: HashMap<String, serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct SessionStore {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_session(&self) -> String {
        let session_id = format!("{}{}", chrono::Utc::now().timestamp(), std::process::id());
        let session = Session {
            id: session_id.clone(),
            data: HashMap::new(),
            created_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(24),
        };

        self.sessions
            .write()
            .await
            .insert(session_id.clone(), session);
        session_id
    }

    pub async fn get_session(&self, session_id: &str) -> Option<Session> {
        self.sessions.read().await.get(session_id).cloned()
    }

    pub async fn update_session(&self, session_id: &str, session: Session) {
        self.sessions
            .write()
            .await
            .insert(session_id.to_string(), session);
    }

    pub async fn delete_session(&self, session_id: &str) {
        self.sessions.write().await.remove(session_id);
    }

    pub async fn cleanup_expired(&self) {
        let now = chrono::Utc::now();
        self.sessions
            .write()
            .await
            .retain(|_, session| session.expires_at > now);
    }
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new()
    }
}

pub static GLOBAL_SESSION_STORE: once_cell::sync::Lazy<SessionStore> =
    once_cell::sync::Lazy::new(SessionStore::new);

pub type SessionData = (Session, Option<SessionUser>);

pub async fn session_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
    tokio::spawn(async {
        GLOBAL_SESSION_STORE.cleanup_expired().await;
    });
    Ok(next.run(request).await)
}

pub async fn extract_session_data(
    headers: &axum::http::HeaderMap,
) -> Result<SessionData, StatusCode> {
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

pub fn extract_session_id_from_headers(
    headers: &axum::http::HeaderMap,
) -> Result<String, StatusCode> {
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
