#[cfg(test)]
mod tests {
    use crate::web::middleware::{Session, GLOBAL_SESSION_STORE};
    use crate::web::models::SessionUser;
    use crate::web::session_extractor::{extract_session_data, extract_session_id_from_headers};
    use axum::http::{header::COOKIE, HeaderMap, StatusCode};
    use chrono::{Duration, Utc};
    use serde_json::json;

    async fn setup_test_session(user_data: Option<SessionUser>) -> String {
        let session_id = "test_session_123".to_string();
        let mut data = std::collections::HashMap::new();

        if let Some(user) = user_data {
            data.insert("user".to_string(), serde_json::to_value(user).unwrap());
        }

        let session = Session {
            id: session_id.clone(),
            data,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::hours(1),
        };

        GLOBAL_SESSION_STORE
            .update_session(&session_id, session)
            .await;
        session_id
    }

    fn create_test_user() -> SessionUser {
        SessionUser {
            user: crate::web::models::DiscordUser {
                id: "123456789".to_string(),
                username: "testuser".to_string(),
                discriminator: "0001".to_string(),
                avatar: Some("avatar_hash".to_string()),
                email: None,
                verified: None,
                global_name: None,
            },
            guilds: vec![crate::web::models::Guild {
                id: "guild1".to_string(),
                name: "Test Guild".to_string(),
                icon: Some("guild_icon".to_string()),
                permissions: "2147483647".to_string(), // Admin permissions
                owner: false,
                features: vec![],
                permissions_new: None,
                banner: None,
                description: None,
                splash: None,
                discovery_splash: None,
                preferred_locale: None,
                approximate_member_count: None,
                approximate_presence_count: None,
            }],
            access_token: "test_access_token".to_string(),
        }
    }

    #[tokio::test]
    async fn test_extract_session_data_success() {
        let user = create_test_user();
        let session_id = setup_test_session(Some(user.clone())).await;

        let mut headers = HeaderMap::new();
        headers.insert(
            COOKIE,
            format!("session_id={}", session_id).parse().unwrap(),
        );

        let result = extract_session_data(&headers).await;
        assert!(result.is_ok());

        let (session, user_data) = result.unwrap();
        assert_eq!(session.id, session_id);
        assert!(user_data.is_some());

        let extracted_user = user_data.unwrap();
        assert_eq!(extracted_user.user.id, "123456789");
        assert_eq!(extracted_user.user.username, "testuser");
    }

    #[tokio::test]
    async fn test_extract_session_data_no_cookie() {
        let headers = HeaderMap::new();
        let result = extract_session_data(&headers).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_extract_session_data_invalid_session() {
        let mut headers = HeaderMap::new();
        headers.insert(COOKIE, "session_id=invalid_session_id".parse().unwrap());

        let result = extract_session_data(&headers).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_extract_session_data_expired_session() {
        let user = create_test_user();
        let session_id = "expired_session_123".to_string();
        let mut data = std::collections::HashMap::new();
        data.insert("user".to_string(), serde_json::to_value(user).unwrap());

        let expired_session = Session {
            id: session_id.clone(),
            data,
            created_at: Utc::now() - Duration::hours(2),
            expires_at: Utc::now() - Duration::hours(1), // Expired
        };

        GLOBAL_SESSION_STORE
            .update_session(&session_id, expired_session)
            .await;

        let mut headers = HeaderMap::new();
        headers.insert(
            COOKIE,
            format!("session_id={}", session_id).parse().unwrap(),
        );

        let result = extract_session_data(&headers).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::UNAUTHORIZED);

        // Verify session was deleted
        let session_check = GLOBAL_SESSION_STORE.get_session(&session_id).await;
        assert!(session_check.is_none());
    }

    #[tokio::test]
    async fn test_extract_session_data_no_user_data() {
        let session_id = setup_test_session(None).await;

        let mut headers = HeaderMap::new();
        headers.insert(
            COOKIE,
            format!("session_id={}", session_id).parse().unwrap(),
        );

        let result = extract_session_data(&headers).await;
        assert!(result.is_ok());

        let (session, user_data) = result.unwrap();
        assert_eq!(session.id, session_id);
        assert!(user_data.is_none());
    }

    #[tokio::test]
    async fn test_extract_session_data_corrupted_user_data() {
        let session_id = "corrupted_session_123".to_string();
        let mut data = std::collections::HashMap::new();
        data.insert("user".to_string(), json!({"invalid": "user_data"}));

        let session = Session {
            id: session_id.clone(),
            data,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::hours(1),
        };

        GLOBAL_SESSION_STORE
            .update_session(&session_id, session)
            .await;

        let mut headers = HeaderMap::new();
        headers.insert(
            COOKIE,
            format!("session_id={}", session_id).parse().unwrap(),
        );

        let result = extract_session_data(&headers).await;
        assert!(result.is_ok());

        let (_session, user_data) = result.unwrap();
        assert!(user_data.is_none()); // Should be None due to deserialization error
    }

    #[test]
    fn test_extract_session_id_from_headers_multiple_cookies() {
        let mut headers = HeaderMap::new();
        headers.insert(
            COOKIE,
            "other_cookie=value; session_id=test123; another=value"
                .parse()
                .unwrap(),
        );

        let session_id = extract_session_id_from_headers(&headers);
        assert!(session_id.is_ok());
        assert_eq!(session_id.unwrap(), "test123");
    }

    #[test]
    fn test_extract_session_id_from_headers_no_session_cookie() {
        let mut headers = HeaderMap::new();
        headers.insert(COOKIE, "other_cookie=value; another=value".parse().unwrap());

        let result = extract_session_id_from_headers(&headers);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_extract_session_id_from_headers_invalid_header() {
        let mut headers = HeaderMap::new();
        // Insert invalid UTF-8 bytes
        headers.insert(
            COOKIE,
            axum::http::HeaderValue::from_bytes(&[0xFF, 0xFE]).unwrap(),
        );

        let result = extract_session_id_from_headers(&headers);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_extract_session_id_with_spaces() {
        let mut headers = HeaderMap::new();
        headers.insert(
            COOKIE,
            "  session_id=test123  ; other=value  ".parse().unwrap(),
        );

        let session_id = extract_session_id_from_headers(&headers);
        assert!(session_id.is_ok());
        assert_eq!(session_id.unwrap(), "test123");
    }

    #[test]
    fn test_extract_session_id_first_match() {
        let mut headers = HeaderMap::new();
        headers.insert(
            COOKIE,
            "session_id=first; session_id=second".parse().unwrap(),
        );

        let session_id = extract_session_id_from_headers(&headers);
        assert!(session_id.is_ok());
        assert_eq!(session_id.unwrap(), "first");
    }

    #[tokio::test]
    async fn test_session_cleanup_on_expiry() {
        let user = create_test_user();
        let session_id = "cleanup_test_session".to_string();
        let mut data = std::collections::HashMap::new();
        data.insert("user".to_string(), serde_json::to_value(user).unwrap());

        let expired_session = Session {
            id: session_id.clone(),
            data,
            created_at: Utc::now() - Duration::hours(1),
            expires_at: Utc::now() - Duration::minutes(1),
        };

        GLOBAL_SESSION_STORE
            .update_session(&session_id, expired_session)
            .await;

        let mut headers = HeaderMap::new();
        headers.insert(
            COOKIE,
            format!("session_id={}", session_id).parse().unwrap(),
        );

        // First call should detect expiry and clean up
        let result = extract_session_data(&headers).await;
        assert!(result.is_err());

        // Verify session was actually removed
        let session_check = GLOBAL_SESSION_STORE.get_session(&session_id).await;
        assert!(session_check.is_none());
    }
}
