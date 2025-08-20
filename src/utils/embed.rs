use anyhow::{Result, anyhow};
use rand::{distributions::Alphanumeric, Rng};
use std::path::PathBuf;
use tokio::fs;
use url::Url;
use tracing::{error, info, warn};

/// Generates a random alphanumeric identifier for embed files
pub fn generate_embed_id(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

/// Validates if a URL is accessible and points to a video file
pub async fn validate_video_url(url: &str) -> Result<Url> {
    // Parse the URL
    let parsed_url = Url::parse(url)
        .map_err(|_| anyhow!("Invalid URL format"))?;
    
    // Check if the scheme is HTTP or HTTPS
    if parsed_url.scheme() != "http" && parsed_url.scheme() != "https" {
        return Err(anyhow!("URL must use HTTP or HTTPS scheme"));
    }
    
    // Check if the URL has a valid host
    if parsed_url.host().is_none() || parsed_url.host_str().map_or(true, |h| h.is_empty()) {
        return Err(anyhow!("URL must have a valid host"));
    }
    
    // Check if the path is not empty (should contain the actual video file)
    if parsed_url.path().is_empty() || parsed_url.path() == "/" {
        return Err(anyhow!("URL must have a valid path to a video file"));
    }
    
    // Basic extension check for common video formats
    let path = parsed_url.path().to_lowercase();
    let valid_extensions = [".mp4", ".webm", ".mov", ".avi", ".mkv", ".m4v"];
    
    let has_valid_extension = valid_extensions.iter().any(|ext| path.ends_with(ext));
    if !has_valid_extension {
        warn!("URL '{}' does not have a recognized video extension", url);
    }
    
    Ok(parsed_url)
}

/// Sanitizes a string for safe use in HTML content
pub fn sanitize_html_content(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Extracts a title from a video URL (uses filename or domain)
pub fn extract_video_title(url: &Url) -> String {
    // Try to get filename from path
    if let Some(filename) = url.path_segments().and_then(|segments| segments.last()) {
        if !filename.is_empty() && filename.contains('.') {
            // Remove extension and decode
            let name = filename.split('.').next().unwrap_or(filename);
            return urlencoding::decode(name)
                .map(|s| s.into_owned())
                .unwrap_or_else(|_| name.to_string())
                .replace('_', " ")
                .replace('-', " ");
        }
    }
    
    // Fallback to domain name
    url.host_str()
        .map(|host| format!("Video from {}", host))
        .unwrap_or_else(|| "Video".to_string())
}

/// Creates the embed directory if it doesn't exist
pub async fn ensure_embed_directory(directory: &str) -> Result<PathBuf> {
    let path = PathBuf::from(directory);
    
    if !path.exists() {
        info!("Creating embed directory: {}", directory);
        fs::create_dir_all(&path).await
            .map_err(|e| anyhow!("Failed to create embed directory '{}': {}", directory, e))?;
    }
    
    Ok(path)
}

/// Generates the HTML content for video embedding
pub fn generate_embed_html(video_url: &str, title: &str, width: u32, height: u32) -> String {
    let sanitized_url = sanitize_html_content(video_url);
    let sanitized_title = sanitize_html_content(title);
    let description = format!("Embedded video: {}", sanitized_title);
    
    // Build the HTML content using concatenation to avoid format string issues
    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
    html.push_str("    <meta charset=\"UTF-8\">\n");
    html.push_str("    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
    html.push_str("    <meta property=\"og:type\" content=\"video.other\">\n");
    html.push_str(&format!("    <meta property=\"og:video:url\" content=\"{}\">\n", sanitized_url));
    html.push_str("    <meta property=\"og:video:type\" content=\"video/mp4\">\n");
    html.push_str(&format!("    <meta property=\"og:video:width\" content=\"{}\">\n", width));
    html.push_str(&format!("    <meta property=\"og:video:height\" content=\"{}\">\n", height));
    html.push_str(&format!("    <meta property=\"og:title\" content=\"{}\">\n", sanitized_title));
    html.push_str(&format!("    <meta property=\"og:description\" content=\"{}\">\n", description));
    html.push_str("    \n");
    html.push_str("    <!-- Twitter Card meta tags for better compatibility -->\n");
    html.push_str("    <meta name=\"twitter:card\" content=\"player\">\n");
    html.push_str(&format!("    <meta name=\"twitter:player\" content=\"{}\">\n", sanitized_url));
    html.push_str(&format!("    <meta name=\"twitter:player:width\" content=\"{}\">\n", width));
    html.push_str(&format!("    <meta name=\"twitter:player:height\" content=\"{}\">\n", height));
    html.push_str("    \n");
    html.push_str("    <!-- Discord-specific optimization -->\n");
    html.push_str("    <meta name=\"theme-color\" content=\"#5865F2\">\n");
    html.push_str("    \n");
    html.push_str(&format!("    <title>{}</title>\n", sanitized_title));
    html.push_str("    <style>\n");
    html.push_str("        body {\n");
    html.push_str("            margin: 0;\n");
    html.push_str("            padding: 20px;\n");
    html.push_str("            font-family: Arial, sans-serif;\n");
    html.push_str("            background-color: #2f3136;\n");
    html.push_str("            color: white;\n");
    html.push_str("            display: flex;\n");
    html.push_str("            flex-direction: column;\n");
    html.push_str("            align-items: center;\n");
    html.push_str("            justify-content: center;\n");
    html.push_str("            min-height: 100vh;\n");
    html.push_str("        }\n");
    html.push_str("        video {\n");
    html.push_str("            max-width: 100%;\n");
    html.push_str("            max-height: 80vh;\n");
    html.push_str("            border-radius: 8px;\n");
    html.push_str("            box-shadow: 0 4px 8px rgba(0, 0, 0, 0.3);\n");
    html.push_str("        }\n");
    html.push_str("        .title {\n");
    html.push_str("            margin-bottom: 15px;\n");
    html.push_str("            font-size: 1.2em;\n");
    html.push_str("            text-align: center;\n");
    html.push_str("        }\n");
    html.push_str("        .fallback {\n");
    html.push_str("            margin-top: 15px;\n");
    html.push_str("            text-align: center;\n");
    html.push_str("        }\n");
    html.push_str("        .fallback a {\n");
    html.push_str("            color: #5865F2;\n");
    html.push_str("            text-decoration: none;\n");
    html.push_str("        }\n");
    html.push_str("        .fallback a:hover {\n");
    html.push_str("            text-decoration: underline;\n");
    html.push_str("        }\n");
    html.push_str("    </style>\n");
    html.push_str("</head>\n");
    html.push_str("<body>\n");
    html.push_str(&format!("    <div class=\"title\">{}</div>\n", sanitized_title));
    html.push_str(&format!("    <video controls width=\"{}\" height=\"{}\" preload=\"metadata\">\n", width, height));
    html.push_str(&format!("        <source src=\"{}\" type=\"video/mp4\">\n", sanitized_url));
    html.push_str("        <p>Your browser does not support the video element.</p>\n");
    html.push_str("    </video>\n");
    html.push_str("    <div class=\"fallback\">\n");
    html.push_str(&format!("        <a href=\"{}\" target=\"_blank\">Open video in new tab</a>\n", sanitized_url));
    html.push_str("    </div>\n");
    html.push_str("</body>\n");
    html.push_str("</html>");
    
    html
}

/// Saves HTML content to a file in the embed directory
pub async fn save_embed_file(
    directory: &str,
    filename: &str,
    content: &str,
) -> Result<PathBuf> {
    // Validate filename first to reject obvious traversal attempts
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(anyhow!("directory traversal attempt blocked"));
    }
    
    let embed_dir = ensure_embed_directory(directory).await?;
    let file_path = embed_dir.join(format!("{}.html", filename));
    
    // Prevent directory traversal by canonicalizing and checking the resolved path
    let canonical_embed_dir = embed_dir.canonicalize()
        .map_err(|e| anyhow!("Failed to canonicalize embed directory: {}", e))?;
    
    // Try to canonicalize the file path, but handle cases where the file doesn't exist yet
    let canonical_file_path = if let Ok(canonical) = file_path.canonicalize() {
        canonical
    } else {
        // If file doesn't exist, canonicalize the parent and join the filename
        if let Some(parent) = file_path.parent() {
            let canonical_parent = parent.canonicalize()
                .map_err(|_| anyhow!("directory traversal attempt blocked"))?;
            if let Some(filename_only) = file_path.file_name() {
                canonical_parent.join(filename_only)
            } else {
                return Err(anyhow!("directory traversal attempt blocked"));
            }
        } else {
            return Err(anyhow!("directory traversal attempt blocked"));
        }
    };
    
    if !canonical_file_path.starts_with(&canonical_embed_dir) {
        return Err(anyhow!("directory traversal attempt blocked"));
    }
    
    fs::write(&file_path, content).await
        .map_err(|e| anyhow!("Failed to write embed file '{}': {}", file_path.display(), e))?;
    
    info!("Created embed file: {}", file_path.display());
    Ok(file_path)
}

/// Cleans up old embed files based on age
pub async fn cleanup_old_embeds(directory: &str, max_age_hours: u64) -> Result<usize> {
    let embed_dir = PathBuf::from(directory);
    
    if !embed_dir.exists() {
        return Ok(0);
    }
    
    let mut entries = fs::read_dir(&embed_dir).await
        .map_err(|e| anyhow!("Failed to read embed directory '{}': {}", directory, e))?;
    
    let max_age = std::time::Duration::from_secs(max_age_hours * 3600);
    let now = std::time::SystemTime::now();
    let mut cleaned_count = 0;
    
    while let Some(entry) = entries.next_entry().await
        .map_err(|e| anyhow!("Failed to read directory entry: {}", e))? {
        
        let path = entry.path();
        
        // Only process .html files
        if path.extension().and_then(|s| s.to_str()) != Some("html") {
            continue;
        }
        
        // Check file age
        if let Ok(metadata) = entry.metadata().await {
            if let Ok(created) = metadata.created() {
                if let Ok(age) = now.duration_since(created) {
                    if age > max_age {
                        if fs::remove_file(&path).await.is_ok() {
                            info!("Cleaned up old embed file: {}", path.display());
                            cleaned_count += 1;
                        } else {
                            error!("Failed to remove old embed file: {}", path.display());
                        }
                    }
                }
            }
        }
    }
    
    Ok(cleaned_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_generate_embed_id() {
        let id = generate_embed_id(8);
        assert_eq!(id.len(), 8);
        assert!(id.chars().all(|c| c.is_alphanumeric()));
        
        // Test different lengths
        for length in [4, 8, 12, 16, 20] {
            let id = generate_embed_id(length);
            assert_eq!(id.len(), length);
            assert!(id.chars().all(|c| c.is_alphanumeric()));
        }
        
        // Test uniqueness
        let mut ids = std::collections::HashSet::new();
        for _ in 0..1000 {
            let id = generate_embed_id(12);
            assert!(ids.insert(id), "Generated duplicate ID");
        }
    }
    
    #[test]
    fn test_sanitize_html_content() {
        let test_cases = vec![
            ("normal text", "normal text"),
            ("<script>alert('xss')</script>", "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;"),
            ("&amp;", "&amp;amp;"),
            ("\"quotes\"", "&quot;quotes&quot;"),
            ("'apostrophes'", "&#x27;apostrophes&#x27;"),
            ("<>&\"'", "&lt;&gt;&amp;&quot;&#x27;"),
            ("", ""),
            ("Mix<ed>&content\"here'", "Mix&lt;ed&gt;&amp;content&quot;here&#x27;"),
        ];
        
        for (input, expected) in test_cases {
            let result = sanitize_html_content(input);
            assert_eq!(result, expected, "Input: {} should sanitize to: {}", input, expected);
        }
    }
    
    #[tokio::test]
    async fn test_validate_video_url() {
        // Valid URLs
        let valid_urls = vec![
            "https://example.com/video.mp4",
            "http://example.com/video.webm",
            "https://sub.example.com/path/video.mov",
            "https://example.com:8080/video.avi",
            "https://example.com/video.mkv",
            "https://example.com/video.m4v",
            "https://example.com/path/to/video.MP4", // uppercase extension
        ];
        
        for url in valid_urls {
            assert!(validate_video_url(url).await.is_ok(), "URL should be valid: {}", url);
        }
        
        // Invalid URLs
        let invalid_urls = vec![
            "ftp://example.com/video.mp4",
            "file:///local/video.mp4", 
            "javascript:alert('xss')",
            "not-a-url",
            "",
            "https://",
            "https:///video.mp4",
        ];
        
        for url in invalid_urls {
            assert!(validate_video_url(url).await.is_err(), "URL should be invalid: {}", url);
        }
        
        // URL without extension (should still pass but with warning)
        assert!(validate_video_url("https://example.com/video").await.is_ok());
    }
    
    #[test]
    fn test_extract_video_title() {
        let test_cases = vec![
            ("https://example.com/my_cool_video.mp4", "my cool video"),
            ("https://example.com/test-video_file.webm", "test video file"),
            ("https://example.com/Video_With_Underscores.mov", "Video With Underscores"),
            ("https://example.com/hyphen-separated-name.avi", "hyphen separated name"),
            ("https://example.com/", "Video from example.com"),
            ("https://example.com/path/", "Video from example.com"),
            ("https://sub.example.com/video.mp4", "video"),
            ("https://example.com/video", "Video from example.com"), // no extension
        ];
        
        for (url_str, expected_title) in test_cases {
            let url = Url::parse(url_str).unwrap();
            let title = extract_video_title(&url);
            assert_eq!(title, expected_title, "URL: {} should extract title: {}", url_str, expected_title);
        }
    }
    
    #[tokio::test]
    async fn test_directory_operations() {
        let temp_dir = TempDir::new().unwrap();
        let embed_dir = temp_dir.path().join("test_embeds");
        let embed_dir_str = embed_dir.to_string_lossy().to_string();
        
        // Test directory creation
        let created_dir = ensure_embed_directory(&embed_dir_str).await.unwrap();
        assert!(created_dir.exists());
        assert!(created_dir.is_dir());
        
        // Test calling again (should not fail)
        let created_dir2 = ensure_embed_directory(&embed_dir_str).await.unwrap();
        assert_eq!(created_dir, created_dir2);
    }
    
    #[tokio::test]
    async fn test_file_operations() {
        let temp_dir = TempDir::new().unwrap();
        let embed_dir = temp_dir.path().join("test_embeds");
        let embed_dir_str = embed_dir.to_string_lossy().to_string();
        
        // Test file saving
        let test_content = "<html>test content</html>";
        let filename = "test_embed";
        
        let file_path = save_embed_file(&embed_dir_str, filename, test_content).await.unwrap();
        assert!(file_path.exists());
        assert!(file_path.file_name().unwrap() == "test_embed.html");
        
        let saved_content = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(saved_content, test_content);
        
        // Test directory traversal protection
        let malicious_filenames = vec![
            "../../../etc/passwd",
            "..\\..\\..\\windows\\system32\\config\\sam",
            "../../../../bin/sh",
            "../outside_directory",
        ];
        
        for malicious_filename in malicious_filenames {
            let result = save_embed_file(&embed_dir_str, malicious_filename, test_content).await;
            assert!(result.is_err(), "Should reject malicious filename: {}", malicious_filename);
            assert!(result.unwrap_err().to_string().contains("directory traversal"));
        }
        
        // Test empty filename
        let result = save_embed_file(&embed_dir_str, "", test_content).await;
        assert!(result.is_ok()); // Will create ".html" file
        
        // Test filename with special characters
        let special_filename = "test@file#with$special%chars";
        let result = save_embed_file(&embed_dir_str, special_filename, test_content).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_html_generation() {
        let video_url = "https://example.com/test.mp4";
        let title = "Test Video";
        let width = 1280u32;
        let height = 720u32;
        
        let html = generate_embed_html(video_url, title, width, height);
        
        // Check basic HTML structure
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("<html lang=\"en\">"));
        assert!(html.contains("</html>"));
        
        // Check Open Graph meta tags
        assert!(html.contains("og:type\" content=\"video.other\""));
        assert!(html.contains(&format!("og:video:url\" content=\"{}\"", video_url)));
        assert!(html.contains("og:video:type\" content=\"video/mp4\""));
        assert!(html.contains(&format!("og:video:width\" content=\"{}\"", width)));
        assert!(html.contains(&format!("og:video:height\" content=\"{}\"", height)));
        assert!(html.contains(&format!("og:title\" content=\"{}\"", title)));
        
        // Check Twitter Card meta tags
        assert!(html.contains("twitter:card\" content=\"player\""));
        assert!(html.contains(&format!("twitter:player\" content=\"{}\"", video_url)));
        
        // Check video element
        assert!(html.contains("<video controls"));
        assert!(html.contains(&format!("width=\"{}\"", width)));
        assert!(html.contains(&format!("height=\"{}\"", height)));
        assert!(html.contains(&format!("src=\"{}\"", video_url)));
        
        // Check fallback link
        assert!(html.contains(&format!("href=\"{}\"", video_url)));
        
        // Check CSS styling
        assert!(html.contains("background-color: #2f3136"));
        assert!(html.contains("color: #5865F2"));
        
        // Check Discord theme color
        assert!(html.contains("theme-color\" content=\"#5865F2\""));
    }
    
    #[tokio::test]
    async fn test_html_sanitization_in_generation() {
        let malicious_url = "https://example.com/test.mp4?param=<script>alert('xss')</script>";
        let malicious_title = "<script>alert('xss')</script>";
        
        let html = generate_embed_html(malicious_url, malicious_title, 1920, 1080);
        
        // Should not contain raw script tags
        assert!(!html.contains("<script>"));
        assert!(!html.contains("alert('xss')"));
        
        // Should contain escaped versions
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains("alert(&#x27;xss&#x27;)"));
    }
    
    #[tokio::test]
    async fn test_cleanup_functionality() {
        let temp_dir = TempDir::new().unwrap();
        let embed_dir = temp_dir.path().join("cleanup_test");
        let embed_dir_str = embed_dir.to_string_lossy().to_string();
        
        // Create directory
        fs::create_dir_all(&embed_dir).await.unwrap();
        
        // Create some test files
        let files = vec![
            ("old_file1.html", "old content 1"),
            ("old_file2.html", "old content 2"),
            ("new_file.html", "new content"),
            ("other_file.txt", "text content"), // non-HTML file
        ];
        
        for (filename, content) in &files {
            let file_path = embed_dir.join(filename);
            fs::write(&file_path, content).await.unwrap();
        }
        
        // Test cleanup with very low max age (should clean all HTML files)
        let cleaned = cleanup_old_embeds(&embed_dir_str, 0).await.unwrap();
        assert!(cleaned >= 3); // Should clean at least the 3 HTML files
        
        // Verify non-HTML file still exists
        let txt_file = embed_dir.join("other_file.txt");
        assert!(txt_file.exists());
        
        // Test cleanup with non-existent directory
        let nonexistent_dir = temp_dir.path().join("nonexistent");
        let result = cleanup_old_embeds(&nonexistent_dir.to_string_lossy(), 24).await.unwrap();
        assert_eq!(result, 0);
    }
    
    #[tokio::test]
    async fn test_concurrent_file_operations() {
        let temp_dir = TempDir::new().unwrap();
        let embed_dir = temp_dir.path().join("concurrent_test");
        let embed_dir_str = embed_dir.to_string_lossy().to_string();
        
        // Test concurrent file creation
        let mut handles = Vec::new();
        for i in 0..10 {
            let dir = embed_dir_str.clone();
            let handle = tokio::spawn(async move {
                let filename = format!("test_file_{}", i);
                let content = format!("<html>Test content {}</html>", i);
                save_embed_file(&dir, &filename, &content).await
            });
            handles.push(handle);
        }
        
        // Wait for all operations to complete
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }
        
        // Verify all files were created
        let mut entries = fs::read_dir(&embed_dir).await.unwrap();
        let mut count = 0;
        while let Some(_entry) = entries.next_entry().await.unwrap() {
            count += 1;
        }
        assert_eq!(count, 10);
    }
    
    #[tokio::test]
    async fn test_edge_cases() {
        let temp_dir = TempDir::new().unwrap();
        let embed_dir_str = temp_dir.path().to_string_lossy().to_string();
        
        // Test with very long content
        let long_content = "x".repeat(10_000);
        let result = save_embed_file(&embed_dir_str, "long_content", &long_content).await;
        assert!(result.is_ok());
        
        // Test with unicode content
        let unicode_content = "🎬 Video embed with unicode 中文 العربية";
        let result = save_embed_file(&embed_dir_str, "unicode_test", unicode_content).await;
        assert!(result.is_ok());
        
        // Test with empty content
        let result = save_embed_file(&embed_dir_str, "empty_test", "").await;
        assert!(result.is_ok());
        
        // Test HTML generation with edge case dimensions
        let html = generate_embed_html("https://example.com/test.mp4", "Test", 100, 100);
        assert!(html.contains("width=\"100\""));
        assert!(html.contains("height=\"100\""));
        
        let html = generate_embed_html("https://example.com/test.mp4", "Test", 4096, 4096);
        assert!(html.contains("width=\"4096\""));
        assert!(html.contains("height=\"4096\""));
    }
}