use anyhow::Result;
use std::path::Path;
use tokio::fs;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn cleanup_old_embeds(directory: &str, max_age_hours: u64) -> Result<()> {
    if !Path::new(directory).exists() {
        return Ok(());
    }
    
    let max_age_seconds = max_age_hours * 3600;
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs();
    
    let mut dir_entries = fs::read_dir(directory).await?;
    
    while let Some(entry) = dir_entries.next_entry().await? {
        let metadata = entry.metadata().await?;
        
        if metadata.is_file() {
            let modified_time = metadata
                .modified()?
                .duration_since(UNIX_EPOCH)?
                .as_secs();
            
            if current_time - modified_time > max_age_seconds {
                if let Err(e) = fs::remove_file(entry.path()).await {
                    tracing::warn!("Failed to remove old embed file: {}", e);
                }
            }
        }
    }
    
    Ok(())
}