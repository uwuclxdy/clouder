#[cfg(test)]
mod tests {
    use crate::config::{AppState, Config};
    use serenity::all::{Cache, Http};
    use std::sync::Arc;
    use tempfile::TempDir;
    use tokio::fs;
    use crate::utils::embed::{extract_video_title, generate_embed_html, generate_embed_id, save_embed_file, validate_video_url};
    use axum::http::{Method, Request, StatusCode};
    use axum::body::Body;
    use axum::Router;
    use tower::ServiceExt;
    use crate::web::create_router;

    async fn setup_test_state() -> (AppState, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let embed_path = temp_dir.path().join("embed_files");
        fs::create_dir_all(&embed_path).await.expect("Failed to create embed directory");

        let mut config = Config::test_config();
        config.web.embed.directory = embed_path.to_string_lossy().to_string();
        config.web.base_url = "http://test.example.com".to_string();

        let db = Arc::new(
            sqlx::SqlitePool::connect(":memory:")
                .await
                .expect("Failed to create test database")
        );

        // Note: In real tests, you might want to run migrations
        // sqlx::migrate!("./migrations").run(&*db).await.expect("Failed to run migrations");

        let cache = Arc::new(Cache::new());
        let http = Arc::new(Http::new("test_token"));

        let app_state = AppState::new(Arc::new(config), db, cache, http);
        (app_state, temp_dir)
    }

    #[tokio::test]
    async fn test_video_command_basic_functionality() {
        let (app_state, _temp_dir) = setup_test_state().await;

        // Test basic video processing
        let video_url = "https://example.com/test_video.mp4";
        let title = "Test Video";
        let width = 1920u32;
        let height = 1080u32;

        // Test URL validation
        let parsed_url = validate_video_url(video_url).await.unwrap();
        assert_eq!(parsed_url.scheme(), "https");
        assert_eq!(parsed_url.host_str().unwrap(), "example.com");

        // Test title extraction
        let extracted_title = extract_video_title(&parsed_url);
        assert_eq!(extracted_title, "test video");

        // Test ID generation
        let embed_id = generate_embed_id(12);
        assert_eq!(embed_id.len(), 12);
        assert!(embed_id.chars().all(|c| c.is_alphanumeric()));

        // Test HTML generation
        let html_content = generate_embed_html(video_url, title, width, height);
        assert!(html_content.contains("<!DOCTYPE html>"));
        assert!(html_content.contains("og:video:url"));
        assert!(html_content.contains(video_url));
        assert!(html_content.contains(title));

        // Test file saving
        let file_path = save_embed_file(&app_state.config.web.embed.directory, &embed_id, &html_content).await.unwrap();
        assert!(file_path.exists());

        // Verify file content
        let saved_content = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(saved_content, html_content);
    }

    #[tokio::test]
    async fn test_dimension_validation() {
        // Test the logic used in the video command
        let validate_dimensions = |w: u32, h: u32| -> bool {
            w >= 100 && w <= 4096 && h >= 100 && h <= 4096
        };

        assert!(!validate_dimensions(99, 1080)); // too small width
        assert!(!validate_dimensions(1920, 99)); // too small height
        assert!(!validate_dimensions(4097, 1080)); // too large width
        assert!(!validate_dimensions(1920, 4097)); // too large height
        assert!(validate_dimensions(1920, 1080)); // valid
        assert!(validate_dimensions(100, 100)); // min valid
        assert!(validate_dimensions(4096, 4096)); // max valid
    }


    async fn setup_test_app() -> (Router, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let embed_path = temp_dir.path().join("embed_files");
        fs::create_dir_all(&embed_path).await.expect("Failed to create embed directory");

        let mut config = Config::test_config();
        config.web.embed.directory = embed_path.to_string_lossy().to_string();
        config.web.base_url = "http://test.example.com".to_string();

        let db = Arc::new(
            sqlx::SqlitePool::connect(":memory:")
                .await
                .expect("Failed to create test database")
        );

        let cache = Arc::new(Cache::new());
        let http = Arc::new(Http::new("test_token"));

        let app_state = AppState::new(Arc::new(config), db, cache, http);
        let app = create_router(app_state);

        (app, temp_dir)
    }

    #[tokio::test]
    async fn test_serve_video_embed_success() {
        let (app, temp_dir) = setup_test_app().await;

        // Create a test HTML file
        let embed_dir = temp_dir.path().join("embed_files");
        let test_file = embed_dir.join("test123.html");
        let test_content = "<html><head><title>Test</title></head><body>Test Video</body></html>";
        fs::write(&test_file, test_content).await.unwrap();

        // Test request
        let request = Request::builder()
            .method(Method::GET)
            .uri("/video/test123.html")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Check status
        assert_eq!(response.status(), StatusCode::OK);

        // Check headers
        let headers = response.headers();
        assert_eq!(headers.get("content-type").unwrap(), "text/html; charset=utf-8");
        assert_eq!(headers.get("access-control-allow-origin").unwrap(), "*");
        assert_eq!(headers.get("access-control-allow-methods").unwrap(), "GET, OPTIONS");
        assert_eq!(headers.get("cache-control").unwrap(), "public, max-age=3600");

        // Check content
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert_eq!(body_str, test_content);
    }

    #[tokio::test]
    async fn test_serve_video_embed_without_html_extension() {
        let (app, temp_dir) = setup_test_app().await;

        // Create a test HTML file
        let embed_dir = temp_dir.path().join("embed_files");
        let test_file = embed_dir.join("test456.html");
        let test_content = "<html>Test without extension</html>";
        fs::write(&test_file, test_content).await.unwrap();

        // Test request without .html extension
        let request = Request::builder()
            .method(Method::GET)
            .uri("/video/test456")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Check content
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert_eq!(body_str, test_content);
    }

    #[tokio::test]
    async fn test_serve_video_embed_not_found() {
        let (app, _temp_dir) = setup_test_app().await;

        // Test request for non-existent file
        let request = Request::builder()
            .method(Method::GET)
            .uri("/video/nonexistent.html")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_serve_video_embed_directory_traversal_protection() {
        let (app, _temp_dir) = setup_test_app().await;

        // Test directory traversal attempts
        let malicious_paths = vec![
            "/video/../../etc/passwd",
            "/video/../../../windows/system32/config/sam",
            "/video/..%2F..%2F..%2Fetc%2Fpasswd", // URL encoded
            "/video/....//....//etc/passwd",
            "/video/..\\..\\..\\etc\\passwd",
        ];

        for path in malicious_paths {
            let request = Request::builder()
                .method(Method::GET)
                .uri(path)
                .body(Body::empty())
                .unwrap();

            let response = app.clone().oneshot(request).await.unwrap();
            // Should return either 400 Bad Request (our validation) or 404 Not Found (Axum's validation)
            // Both are acceptable security responses for malicious paths
            assert!(
                response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::NOT_FOUND,
                "Path should be blocked with 400 or 404, got {}: {}", response.status(), path
            );
        }
    }

    #[tokio::test]
    async fn test_serve_video_embed_special_characters() {
        let (app, temp_dir) = setup_test_app().await;

        // Create test files with special characters
        let embed_dir = temp_dir.path().join("embed_files");
        let files_to_test = vec![
            ("test_file", "<html>Underscore file</html>"),
            ("test-file", "<html>Hyphen file</html>"),
            ("test123", "<html>Number file</html>"),
        ];

        for (filename, content) in &files_to_test {
            let test_file = embed_dir.join(format!("{}.html", filename));
            fs::write(&test_file, content).await.unwrap();

            let request = Request::builder()
                .method(Method::GET)
                .uri(&format!("/video/{}", filename))
                .body(Body::empty())
                .unwrap();

            let response = app.clone().oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK, "Failed for filename: {}", filename);

            let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let body_str = String::from_utf8(body.to_vec()).unwrap();
            assert_eq!(&body_str, content);
        }
    }

    #[tokio::test]
    async fn test_serve_video_embed_cors_headers() {
        let (app, temp_dir) = setup_test_app().await;

        // Create a test file
        let embed_dir = temp_dir.path().join("embed_files");
        let test_file = embed_dir.join("cors_test.html");
        fs::write(&test_file, "<html>CORS test</html>").await.unwrap();

        let request = Request::builder()
            .method(Method::GET)
            .uri("/video/cors_test.html")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let headers = response.headers();

        // Check all required CORS headers
        assert_eq!(headers.get("access-control-allow-origin").unwrap(), "*");
        assert_eq!(headers.get("access-control-allow-methods").unwrap(), "GET, OPTIONS");
        assert_eq!(headers.get("access-control-allow-headers").unwrap(), "Content-Type");
    }

    #[tokio::test]
    async fn test_serve_video_embed_content_type() {
        let (app, temp_dir) = setup_test_app().await;

        // Create a test file
        let embed_dir = temp_dir.path().join("embed_files");
        let test_file = embed_dir.join("content_type_test.html");
        fs::write(&test_file, "<html>Content type test</html>").await.unwrap();

        let request = Request::builder()
            .method(Method::GET)
            .uri("/video/content_type_test.html")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let content_type = response.headers().get("content-type").unwrap();
        assert_eq!(content_type, "text/html; charset=utf-8");
    }

    #[tokio::test]
    async fn test_serve_video_embed_cache_headers() {
        let (app, temp_dir) = setup_test_app().await;

        // Create a test file
        let embed_dir = temp_dir.path().join("embed_files");
        let test_file = embed_dir.join("cache_test.html");
        fs::write(&test_file, "<html>Cache test</html>").await.unwrap();

        let request = Request::builder()
            .method(Method::GET)
            .uri("/video/cache_test.html")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let cache_control = response.headers().get("cache-control").unwrap();
        assert_eq!(cache_control, "public, max-age=3600");
    }

    #[tokio::test]
    async fn test_serve_video_embed_large_file() {
        let (app, temp_dir) = setup_test_app().await;

        // Create a large test file
        let embed_dir = temp_dir.path().join("embed_files");
        let test_file = embed_dir.join("large_test.html");
        let large_content = format!("<html><body>{}</body></html>", "x".repeat(100_000));
        fs::write(&test_file, &large_content).await.unwrap();

        let request = Request::builder()
            .method(Method::GET)
            .uri("/video/large_test.html")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert_eq!(body_str, large_content);
    }

    #[tokio::test]
    async fn test_serve_video_embed_unicode_content() {
        let (app, temp_dir) = setup_test_app().await;

        // Create a test file with unicode content
        let embed_dir = temp_dir.path().join("embed_files");
        let test_file = embed_dir.join("unicode_test.html");
        let unicode_content = "<html><body>🎬 Video embed 中文 العربية ñoño</body></html>";
        fs::write(&test_file, unicode_content).await.unwrap();

        let request = Request::builder()
            .method(Method::GET)
            .uri("/video/unicode_test.html")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert_eq!(body_str, unicode_content);
    }
}
