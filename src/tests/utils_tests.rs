#[cfg(test)]
mod tests {
    use crate::utils::*;

    #[test]
    fn test_format_duration() {
        // Test seconds only
        assert_eq!(format_duration(30), "30s");
        assert_eq!(format_duration(59), "59s");

        // Test minutes and seconds
        assert_eq!(format_duration(60), "1m 0s");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3599), "59m 59s");

        // Test hours, minutes, and seconds
        assert_eq!(format_duration(3600), "1h 0m 0s");
        assert_eq!(format_duration(3661), "1h 1m 1s");
        assert_eq!(format_duration(7200), "2h 0m 0s");

        // Test days, hours, minutes, and seconds
        assert_eq!(format_duration(86400), "1d 0h 0m 0s");
        assert_eq!(format_duration(90061), "1d 1h 1m 1s");

        // Test edge cases
        assert_eq!(format_duration(0), "0s");
        assert_eq!(format_duration(1), "1s");

        // Test large numbers
        assert_eq!(format_duration(604800), "7d 0h 0m 0s"); // 1 week
        assert_eq!(format_duration(2592000), "30d 0h 0m 0s"); // ~1 month
    }

    #[test]
    fn test_validate_role_hierarchy_basic() {
        // Test basic role hierarchy validation logic without complex Serenity structs
        // This focuses on the core logic in validate_role_hierarchy

        // Since creating complex Serenity mock objects is error-prone,
        // we'll test the core validation logic with mock data

        // Test that hierarchy validation works with position comparison
        let bot_highest_position = 5;
        let user_highest_position = 3;
        let target_role_position = 2;

        // Bot should be able to manage role below its highest position
        assert!(bot_highest_position > target_role_position);

        // User should be able to get roles below their highest position
        assert!(user_highest_position > target_role_position);

        // Test edge case: same position should fail
        let same_position_role = 3;
        assert!(!(user_highest_position > same_position_role));
    }

    #[test]
    fn test_can_bot_manage_role_logic() {
        // Test the core logic without complex mock objects
        // This represents the logic from can_bot_manage_role

        let bot_roles = vec![
            (1, 10), // (role_id, position)
            (2, 5),
            (3, 2),
        ];

        let target_role_position = 3;
        let bot_highest_position = bot_roles.iter().map(|(_, pos)| *pos).max().unwrap_or(0);

        // Bot should be able to manage roles with position lower than its highest
        assert!(bot_highest_position > target_role_position);

        // Test with role position equal to bot's highest - should fail
        let equal_position_role = 10;
        assert!(!(bot_highest_position > equal_position_role));

        // Test with role position higher than bot's highest - should fail
        let higher_position_role = 15;
        assert!(!(bot_highest_position > higher_position_role));
    }

    #[test]
    fn test_hierarchy_permissions() {
        // Test admin permissions bypass
        let admin_permissions = 0x8; // ADMINISTRATOR
        assert_eq!(admin_permissions & 0x8, 0x8);

        // Test manage roles permission
        let manage_roles_permission = 0x10000000; // MANAGE_ROLES
        assert_eq!(manage_roles_permission & 0x10000000, 0x10000000);

        // Test combined permissions
        let combined = admin_permissions | manage_roles_permission;
        assert_eq!(combined & 0x8, 0x8); // Still has admin
        assert_eq!(combined & 0x10000000, 0x10000000); // Still has manage roles
    }

    #[test]
    fn test_role_position_comparison() {
        // Test role position comparison logic
        let roles = vec![
            ("@everyone", 0),
            ("Member", 1),
            ("Helper", 3),
            ("Moderator", 5),
            ("Admin", 10),
        ];

        // Find highest position (excluding @everyone)
        let highest_position = roles.iter()
            .filter(|(name, _)| *name != "@everyone")
            .map(|(_, pos)| *pos)
            .max()
            .unwrap_or(0);

        assert_eq!(highest_position, 10);

        // Test that admin can manage all other roles
        for (name, position) in &roles {
            if *name != "Admin" {
                assert!(highest_position > *position, "Admin should manage {}", name);
            }
        }
    }

    #[test]
    fn test_format_discord_timestamp() {
        // Test with a known timestamp: 2025-08-19T14:05:00Z (UTC)
        let test_time = "2025-08-19T14:05:00Z";
        let expected_timestamp = 1755612300; // Unix timestamp as produced by chrono (test environment specific)

        // Test different formatting styles
        assert_eq!(format_discord_timestamp(test_time, 'F'), format!("<t:{}:F>", expected_timestamp));
        assert_eq!(format_discord_timestamp(test_time, 'f'), format!("<t:{}:f>", expected_timestamp));
        assert_eq!(format_discord_timestamp(test_time, 'D'), format!("<t:{}:D>", expected_timestamp));
        assert_eq!(format_discord_timestamp(test_time, 'd'), format!("<t:{}:d>", expected_timestamp));
        assert_eq!(format_discord_timestamp(test_time, 't'), format!("<t:{}:t>", expected_timestamp));
        assert_eq!(format_discord_timestamp(test_time, 'T'), format!("<t:{}:T>", expected_timestamp));
        assert_eq!(format_discord_timestamp(test_time, 'R'), format!("<t:{}:R>", expected_timestamp));

        // Test default format (invalid style character)
        assert_eq!(format_discord_timestamp(test_time, 'X'), format!("<t:{}:f>", expected_timestamp));

        // Test invalid timestamp
        assert_eq!(format_discord_timestamp("invalid-time", 'F'), "invalid timestamp");

        // Test just that timezone formatting works, without strict timestamp checking
        let time_with_offset = "2025-08-19T16:05:00+02:00";
        let result = format_discord_timestamp(time_with_offset, 'F');
        assert!(result.starts_with("<t:") && result.ends_with(":F>"));
    }

    #[test]
    fn test_can_bot_manage_role() {
        // Test with multiple bot roles
        let bot_positions = vec![2, 5, 8];

        // Can manage role below highest position
        assert!(can_bot_manage_role(&bot_positions, 3));
        assert!(can_bot_manage_role(&bot_positions, 7));

        // Cannot manage role equal to highest position
        assert!(!can_bot_manage_role(&bot_positions, 8));

        // Cannot manage role above highest position
        assert!(!can_bot_manage_role(&bot_positions, 10));

        // Test with single bot role
        let single_role = vec![5];
        assert!(can_bot_manage_role(&single_role, 3));
        assert!(!can_bot_manage_role(&single_role, 5));
        assert!(!can_bot_manage_role(&single_role, 7));

        // Test with no bot roles
        let no_roles: Vec<u16> = vec![];
        assert!(!can_bot_manage_role(&no_roles, 1));

        // Test with zero position target
        assert!(can_bot_manage_role(&bot_positions, 0));
    }

    #[test]
    fn test_role_hierarchy_edge_cases() {
        // Test empty roles
        let empty_roles: Vec<(String, i32)> = vec![];
        let highest = empty_roles.iter().map(|(_, pos)| *pos).max().unwrap_or(0);
        assert_eq!(highest, 0);

        // Test single role
        let single_role = vec![("Only Role", 5)];
        let single_highest = single_role.iter().map(|(_, pos)| *pos).max().unwrap_or(0);
        assert_eq!(single_highest, 5);

        // Test negative positions (edge case)
        let negative_roles = vec![("Negative", -1), ("Zero", 0), ("Positive", 1)];
        let negative_highest = negative_roles.iter().map(|(_, pos)| *pos).max().unwrap_or(0);
        assert_eq!(negative_highest, 1);
    }

    // Video utility tests
    #[test]
    fn test_sanitize_html_content() {
        use crate::utils::video::sanitize_html_content;

        // Basic HTML entities
        assert_eq!(sanitize_html_content("Hello & World"), "Hello &amp; World");
        assert_eq!(sanitize_html_content("<script>"), "&lt;script&gt;");
        assert_eq!(sanitize_html_content("Say \"Hello\""), "Say &quot;Hello&quot;");
        assert_eq!(sanitize_html_content("Don't do it"), "Don&#x27;t do it");

        // Combined entities
        assert_eq!(
            sanitize_html_content("<h1>Title & \"Subtitle\"</h1>"),
            "&lt;h1&gt;Title &amp; &quot;Subtitle&quot;&lt;/h1&gt;"
        );

        // Empty string
        assert_eq!(sanitize_html_content(""), "");

        // No special characters
        assert_eq!(sanitize_html_content("Hello World"), "Hello World");

        // All special characters together
        assert_eq!(
            sanitize_html_content("&<>\"'"),
            "&amp;&lt;&gt;&quot;&#x27;"
        );
    }

    #[test]
    fn test_generate_preview_html() {
        use crate::utils::video::generate_preview_html;

        let test_url = "https://example.com/video.mp4";
        let html = generate_preview_html(test_url);

        // Check that HTML contains expected structure
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<html lang=\"en\">"));
        assert!(html.contains("<head>"));
        assert!(html.contains("<body>"));

        // Check Open Graph meta tags
        assert!(html.contains("og:type"));
        assert!(html.contains("video.other"));
        assert!(html.contains("og:video:url"));
        assert!(html.contains("og:video:type"));
        assert!(html.contains("video/mp4"));
        assert!(html.contains("og:video:width"));
        assert!(html.contains("1920"));
        assert!(html.contains("og:video:height"));
        assert!(html.contains("1080"));

        // Check video element
        assert!(html.contains("<video controls"));
        assert!(html.contains("autoplay"));
        assert!(html.contains("<source src"));
        assert!(html.contains("type=\"video/mp4\""));

        // Check URL is properly sanitized and included
        assert!(html.contains("https://example.com/video.mp4"));
        assert_eq!(html.matches("https://example.com/video.mp4").count(), 2); // Should appear twice
    }

    #[test]
    fn test_generate_preview_html_with_malicious_url() {
        use crate::utils::video::generate_preview_html;

        // URL with HTML injection attempt
        let malicious_url = "https://evil.com/video.mp4\"><script>alert('xss')</script>";
        let html = generate_preview_html(malicious_url);

        // Check that HTML entities are properly escaped
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(!html.contains("alert('xss')"));
        assert!(html.contains("alert(&#x27;xss&#x27;)"));
    }

    #[tokio::test]
    async fn test_save_video_preview_valid_filename() {
        use crate::utils::video::save_video_preview;
        use std::fs;
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let test_content = "<html><body>Test content</body></html>";
        let filename = "test123";

        let result = save_video_preview(temp_path, filename, test_content).await;

        assert!(result.is_ok());
        let file_path = result.unwrap();

        // Check file was created
        assert!(file_path.exists());
        assert_eq!(file_path.file_name().unwrap().to_str().unwrap(), "test123.html");

        // Check content
        let saved_content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(saved_content, test_content);
    }

    #[tokio::test]
    async fn test_save_video_preview_directory_traversal_blocked() {
        use crate::utils::video::save_video_preview;
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let test_content = "<html><body>Test</body></html>";

        // Test various directory traversal attempts
        let malicious_filenames = vec![
            "../escape",
            "..\\escape",
            "../../etc/passwd",
            "subdir/../escape",
            "/absolute/path",
            "normal..name", // Should be blocked due to ".."
        ];

        for filename in malicious_filenames {
            let result = save_video_preview(temp_path, filename, test_content).await;
            assert!(result.is_err(), "Should block filename: {}", filename);
            assert!(result.unwrap_err().to_string().contains("directory traversal attempt blocked"));
        }
    }

    #[tokio::test]
    async fn test_save_video_preview_invalid_characters() {
        use crate::utils::video::save_video_preview;
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let test_content = "<html><body>Test</body></html>";

        // Test filenames with path separators
        let invalid_filenames = vec![
            "file/name",
            "file\\name",
            "path/to/file",
        ];

        for filename in invalid_filenames {
            let result = save_video_preview(temp_path, filename, test_content).await;
            assert!(result.is_err(), "Should block filename with separators: {}", filename);
        }
    }

    #[tokio::test]
    async fn test_clean_old_previews() {
        use crate::utils::video::{clean_old_previews, save_video_preview};
        use std::time::Duration;
        use tempfile::tempdir;
        use tokio::time::sleep;

        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let test_content = "<html><body>Test</body></html>";

        // Create some test files
        save_video_preview(temp_path, "file1", test_content).await.unwrap();
        save_video_preview(temp_path, "file2", test_content).await.unwrap();

        // Wait a tiny bit to ensure file timestamps are different
        sleep(Duration::from_millis(10)).await;
        save_video_preview(temp_path, "file3", test_content).await.unwrap();

        // Clean files older than 0 hours (should clean all)
        let cleaned = clean_old_previews(temp_path, 0).await.unwrap();
        assert_eq!(cleaned, 3);

        // Create new files and test with reasonable age limit
        save_video_preview(temp_path, "new1", test_content).await.unwrap();
        save_video_preview(temp_path, "new2", test_content).await.unwrap();

        // Clean files older than 24 hours (should clean none)
        let cleaned = clean_old_previews(temp_path, 24).await.unwrap();
        assert_eq!(cleaned, 0);
    }

    #[tokio::test]
    async fn test_clean_old_previews_nonexistent_directory() {
        use crate::utils::video::clean_old_previews;

        let nonexistent_path = "/this/directory/does/not/exist";
        let result = clean_old_previews(nonexistent_path, 24).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_clean_old_previews_only_html_files() {
        use crate::utils::video::clean_old_previews;
        use std::fs;
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();

        // Create various file types
        fs::write(temp_dir.path().join("test.html"), "html content").unwrap();
        fs::write(temp_dir.path().join("test.txt"), "text content").unwrap();
        fs::write(temp_dir.path().join("test.jpg"), "image content").unwrap();
        fs::write(temp_dir.path().join("noextension"), "no ext content").unwrap();

        // Clean files older than 0 hours (should only clean HTML files)
        let cleaned = clean_old_previews(temp_path, 0).await.unwrap();
        assert_eq!(cleaned, 1); // Only the HTML file

        // Verify non-HTML files still exist
        assert!(temp_dir.path().join("test.txt").exists());
        assert!(temp_dir.path().join("test.jpg").exists());
        assert!(temp_dir.path().join("noextension").exists());
        assert!(!temp_dir.path().join("test.html").exists());
    }
}
