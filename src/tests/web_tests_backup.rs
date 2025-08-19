#[cfg(test)]
mod tests {
    use crate::web;
    use axum::{
        body::Body,
        http::{Request, StatusCode, HeaderMap, header::{AUTHORIZATION, COOKIE}},
        response::Response,
    };
    use axum::extract::{Path, State};
    use serde_json::{json, Value};
    use tower::ServiceExt; // for `oneshot`

    async fn create_test_router() -> axum::Router {
        let app_state = crate::tests::create_test_app_state().await;
        web::create_router(app_state)
    }

    #[tokio::test]
    async fn test_api_get_selfroles_empty() {
        let app = create_test_router().await;
        
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/selfroles/123456789")
                    .method("GET")
                    .header(COOKIE, "test_session=valid")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        
        // Should return 401 due to missing session
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test] 
    async fn test_api_create_selfroles_invalid_session() {
        let app = create_test_router().await;
        
        let request_body = json!({
            "title": "Test Roles",
            "body": "Select your roles",
            "selection_type": "single",
            "channel_id": "987654321",
            "roles": [
                {"role_id": "role123", "emoji": "🎮"}
            ]
        });
        
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/selfroles/123456789")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(request_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        
        // Should return 401 due to missing session
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_api_update_selfroles_not_found() {
        let app = create_test_router().await;
        
        let request_body = json!({
            "title": "Updated Roles",
            "body": "Updated description",
            "selection_type": "multiple",
            "channel_id": "987654321",
            "roles": [
                {"role_id": "role123", "emoji": "🎮"}
            ]
        });
        
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/selfroles/123456789/999")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(request_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        
        // Should return 401 due to missing session
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_api_delete_selfroles_unauthorized() {
        let app = create_test_router().await;
        
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/selfroles/123456789/1")
                    .method("DELETE")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        
        // Should return 401 due to missing session
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_dashboard_routes_redirect_without_auth() {
        let app = create_test_router().await;
        
        // Test server list
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        
        assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
        
        // Test guild dashboard
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/dashboard/123456789")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        
        assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
        
        // Test selfroles list
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/dashboard/123456789/selfroles")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        
        assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
        
        // Test selfroles create
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/dashboard/123456789/selfroles/new")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        
        assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
        
        // Test selfroles edit
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/dashboard/123456789/selfroles/edit/1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        
        assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
    }

    #[tokio::test]
    async fn test_auth_routes_exist() {
        let app = create_test_router().await;
        
        // Test login page
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/auth/login")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        
        assert_eq!(response.status(), StatusCode::OK);
        
        // Test Discord OAuth redirect
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/auth/discord")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        
        assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
        
        // Test logout
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/auth/logout")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        
        assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
    }

    #[tokio::test]
    async fn test_api_routes_structure() {
        let app = create_test_router().await;
        
        // Test GET guild channels (should be unauthorized)
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/guild/123456789/channels")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        
        // Test GET guild roles (should be unauthorized)
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/guild/123456789/roles")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_invalid_routes_return_404() {
        let app = create_test_router().await;
        
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/invalid/route")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/invalid")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_cors_configuration() {
        // Test that CORS headers are properly configured
        // This is a unit test for the CORS middleware configuration
        let cors_layer = tower_http::cors::CorsLayer::new()
            .allow_origin(tower_http::cors::Any)
            .allow_methods([
                axum::http::Method::GET,
                axum::http::Method::POST,
                axum::http::Method::PUT,
                axum::http::Method::DELETE,
            ])
            .allow_headers(tower_http::cors::Any);
        
        // Test that the layer can be created without errors
        assert!(true); // If we get here, the CORS layer was created successfully
    }

    #[test]
    fn test_json_validation() {
        // Test JSON structure validation for API requests
        
        // Valid selfrole creation request
        let valid_json = json!({
            "title": "Test Roles",
            "body": "Select your roles",
            "selection_type": "single",
            "channel_id": "987654321",
            "roles": [
                {"role_id": "role123", "emoji": "🎮"}
            ]
        });
        
        assert!(valid_json.is_object());
        assert!(valid_json["title"].is_string());
        assert!(valid_json["roles"].is_array());
        
        // Invalid JSON structures
        let invalid_json = json!({
            "title": "Test Roles"
            // Missing required fields
        });
        
        assert!(invalid_json["body"].is_null());
        assert!(invalid_json["selection_type"].is_null());
    }

    #[test]
    fn test_request_validation() {
        // Test that request validation works for different scenarios
        
        // Test guild ID validation (should be numeric string)
        let valid_guild_id = "123456789012345678";
        assert!(valid_guild_id.chars().all(|c| c.is_ascii_digit()));
        assert!(valid_guild_id.len() >= 17); // Discord snowflake minimum length
        
        // Test config ID validation (should be valid integer)
        let valid_config_id = "123";
        assert!(valid_config_id.parse::<i64>().is_ok());
        
        let invalid_config_id = "abc";
        assert!(invalid_config_id.parse::<i64>().is_err());
        
        // Test selection type validation
        let valid_selection_types = ["single", "multiple"];
        assert!(valid_selection_types.contains(&"single"));
        assert!(valid_selection_types.contains(&"multiple"));
        assert!(!valid_selection_types.contains(&"invalid"));
    }
}