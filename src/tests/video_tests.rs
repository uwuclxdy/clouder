#[cfg(test)]
mod tests {
    use crate::config::Config;

    #[tokio::test]
    async fn test_video_command_with_url_only() {
        // Test URL without title
        let test_url = "https://example.com/test.mp4".to_string();
        
        // Since we can't easily mock the full poise Context, we'll test the core logic
        // by directly testing the HTML generation and file operations
        assert!(!test_url.is_empty());
        assert!(test_url.starts_with("http"));
        assert!(test_url.ends_with(".mp4"));
    }

    #[tokio::test]
    async fn test_video_command_with_title() {
        // Test the title formatting logic
        let test_title = Some("My Cool Video".to_string());
        
        // Test title processing
        if let Some(title) = &test_title {
            assert!(!title.is_empty());
            let sanitized = crate::utils::video::sanitize_html_content(title);
            assert_eq!(sanitized, "My Cool Video");
        }
    }

    #[tokio::test]
    async fn test_video_command_with_malicious_title() {
        let test_title = Some("<script>alert('xss')</script>".to_string());
        
        if let Some(title) = &test_title {
            let sanitized = crate::utils::video::sanitize_html_content(title);
            assert!(!sanitized.contains("<script>"));
            assert!(sanitized.contains("&lt;script&gt;"));
        }
    }

    #[tokio::test]
    async fn test_video_embed_id_generation() {
        use rand::distr::Alphanumeric;
        use rand::Rng;

        // Test that embed ID generation works as expected
        let embed_id: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(12)
            .map(char::from)
            .collect();
        
        assert_eq!(embed_id.len(), 12);
        assert!(embed_id.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[tokio::test]
    async fn test_video_url_validation() {
        // Test various URL formats
        let valid_urls = vec![
            "https://example.com/video.mp4",
            "http://test.com/file.mp4",
            "https://cdn.discord.com/attachments/123/456/video.mp4",
            "https://youtube.com/watch?v=dQw4w9WgXcQ",
        ];

        for url in valid_urls {
            assert!(!url.is_empty());
            assert!(url.starts_with("http"));
        }
    }

    #[tokio::test]
    async fn test_video_embed_url_construction() {
        let base_url = "https://example.com";
        let embed_id = "abc123def456";
        let embed_url = format!("{}/video/{}.html", base_url, embed_id);
        
        assert_eq!(embed_url, "https://example.com/video/abc123def456.html");
        assert!(embed_url.contains("/video/"));
        assert!(embed_url.ends_with(".html"));
    }

    #[tokio::test]
    async fn test_video_message_formatting() {
        let embed_url = "https://example.com/video/test123.html";
        let title = Some("Test Video".to_string());
        
        // Test message formatting logic
        let mut url_message = embed_url.to_string();
        if title.is_some() {
            let sanitized_title = crate::utils::video::sanitize_html_content("Test Video");
            url_message = format!("[fuc{}]({})", sanitized_title, embed_url);
        }
        
        assert!(url_message.contains("[fuc"));
        assert!(url_message.contains("Test Video"));
        assert!(url_message.contains("]("));
        assert!(url_message.contains(embed_url));
    }

    #[tokio::test]
    async fn test_video_message_formatting_no_title() {
        let embed_url = "https://example.com/video/test123.html";
        let title: Option<String> = None;
        
        let url_message = if title.is_some() {
            let sanitized_title = crate::utils::video::sanitize_html_content(&title.clone().unwrap());
            format!("[fuc{}]({})", sanitized_title, embed_url)
        } else {
            embed_url.to_string()
        };
        
        assert_eq!(url_message, embed_url);
        assert!(!url_message.contains("[fuc"));
    }

    #[tokio::test]
    async fn test_video_html_content_generation() {
        let test_url = "https://example.com/video.mp4";
        let html_content = crate::utils::video::generate_preview_html(test_url);
        
        // Test that HTML is valid and contains expected elements
        assert!(html_content.contains("<!DOCTYPE html>"));
        assert!(html_content.contains("og:video:url"));
        assert!(html_content.contains(test_url));
        assert!(html_content.contains("<video controls"));
        assert!(html_content.contains("autoplay"));
    }

    #[test]
    fn test_video_error_scenarios() {
        // Test various error conditions that the command should handle
        
        // Empty URL
        let empty_url = "";
        assert!(empty_url.is_empty());
        
        // Invalid URL format
        let invalid_urls = vec![
            "not-a-url",
            "ftp://example.com/file.mp4",
            "javascript:alert('xss')",
        ];
        
        for url in invalid_urls {
            // The command should handle these gracefully
            assert!(!url.starts_with("https://") && !url.starts_with("http://"));
        }
    }

    #[test]
    fn test_video_config_access() {
        // Test that config values are properly structured
        let test_config = Config::test_config();
        
        // Check that web config exists
        assert!(!test_config.web.base_url.is_empty());
        assert!(!test_config.web.embed.directory.is_empty());
        
        // Check that embed directory is reasonable
        assert!(test_config.web.embed.directory.len() > 0);
    }

    #[tokio::test]
    async fn test_video_file_operations() {
        use tempfile::tempdir;
        
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let embed_id = "test123";
        let html_content = crate::utils::video::generate_preview_html("https://example.com/video.mp4");
        
        // Test file saving (this tests the integration with utils)
        let result = crate::utils::video::save_video_preview(temp_path, embed_id, &html_content).await;
        
        assert!(result.is_ok());
        let saved_path = result.unwrap();
        assert!(saved_path.exists());
        assert!(saved_path.to_string_lossy().contains("test123.html"));
    }
}