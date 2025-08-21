use anyhow::{anyhow, Result};
use rand::distr::Alphanumeric;
use rand::Rng;
use std::path::PathBuf;
use tokio::fs;
use tracing::{error, info};

/// Generates a random alphanumeric identifier for embed files
pub fn generate_embed_id(length: usize) -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
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
pub fn generate_embed_html(video_url: &str) -> String {
    let sanitized_url = sanitize_html_content(video_url);

    // todo: extract video dimensions from metadata of original file / url
    format!(
        "<!DOCTYPE html>
<html lang=\"en\">
<head>
<meta name=\"viewport\" content=\"width=device-width\">
<meta property=\"og:type\" content=\"video.other\">
<meta property=\"og:video:url\" content=\"{}\">
<meta property=\"og:video:type\" content=\"video/mp4\">
<meta property=\"og:video:width\" content=\"1920\">
<meta property=\"og:video:height\" content=\"1080\">
</head>
<body>
<video controls=\"\" autoplay=\"\" name=\"media\"><source src=\"{}\" type=\"video/mp4\"></video>
</body>
</html>",
        sanitized_url, sanitized_url
    )
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
