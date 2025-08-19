use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use crate::web::models::SessionUser;

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
        let session_id = Uuid::new_v4().to_string();
        let session = Session {
            id: session_id.clone(),
            data: HashMap::new(),
            created_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(24),
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session);
        session_id
    }

    pub async fn get_session(&self, session_id: &str) -> Option<Session> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned()
    }

    pub async fn update_session(&self, session_id: &str, session: Session) {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.to_string(), session);
    }

    pub async fn delete_session(&self, session_id: &str) {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id);
    }

    pub async fn cleanup_expired(&self) {
        let now = chrono::Utc::now();
        let mut sessions = self.sessions.write().await;
        sessions.retain(|_, session| session.expires_at > now);
    }
    
    pub async fn session_count(&self) -> usize {
        self.sessions.read().await.len()
    }
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new()
    }
}

// Global session store instance
lazy_static::lazy_static! {
    pub static ref GLOBAL_SESSION_STORE: SessionStore = SessionStore::new();
}

pub type SessionData = (Session, Option<SessionUser>);

pub async fn session_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Cleanup expired sessions periodically
    tokio::spawn(async {
        GLOBAL_SESSION_STORE.cleanup_expired().await;
    });

    let response = next.run(request).await;
    Ok(response)
}