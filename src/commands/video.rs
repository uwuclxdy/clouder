use crate::config::AppState;
use crate::utils::embed::{
    validate_video_url, extract_video_title, generate_embed_id,
    generate_embed_html, save_embed_file, cleanup_old_embeds
};
use tracing::{error, info};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, AppState, Error>;

/// Create a video embed that bypasses Discord's 10MB limit
#[poise::command(
    slash_command,
    guild_only = true
)]
pub async fn video(
    ctx: Context<'_>,
    #[description = "Direct URL to the video file (e.g., https://example.com/video.mp4)"] url: String,
    #[description = "Custom title for the video (optional)"] title: Option<String>,
    #[description = "Video width in pixels (default: 1920)"] width: Option<u32>,
    #[description = "Video height in pixels (default: 1080)"] height: Option<u32>,
) -> Result<(), Error> {
    ctx.defer().await?;

    let parsed_url = match validate_video_url(&url).await {
        Ok(url) => url,
        Err(e) => {
            ctx.say(format!("❌ invalid video url: {}", e)).await?;
            return Ok(());
        }
    };

    let video_title = title.unwrap_or_else(|| extract_video_title(&parsed_url));

    let video_width = width.unwrap_or(1920);
    let video_height = height.unwrap_or(1080);

    if video_width < 100 || video_width > 4096 || video_height < 100 || video_height > 4096 {
        ctx.say("❌ invalid dimensions: width and height must be between 100 and 4096 pixels").await?;
        return Ok(());
    }

    let embed_id = generate_embed_id(12);
    let html_content = generate_embed_html(&url, &video_title, video_width, video_height);

    // Save HTML file
    let embed_dir = &ctx.data().config.web.embed.directory;
    match save_embed_file(embed_dir, &embed_id, &html_content).await {
        Ok(_) => {
            let base_url = &ctx.data().config.web.base_url;
            let embed_url = format!("{}/video/{}.html", base_url, embed_id);

            ctx.say(embed_url).await?;

            info!("Created video embed {} for URL: {}", embed_id, url);
        }
        Err(e) => {
            error!("Failed to save embed file: {}", e);
            ctx.say("❌ failed to create embed file. please try again later.").await?;
        }
    }

    Ok(())
}

/// Show help information for the video command
#[poise::command(
    slash_command
)]
pub async fn video_help(ctx: Context<'_>) -> Result<(), Error> {
    let help_text = r#"📹 **Video Embed Help**

**How it works:**
The bot creates an HTML page with special meta tags that Discord recognizes as a video embed. This allows you to share videos larger than 10MB as proper embeds.

**Usage:**
`/video url: <video_url> [title: <custom_title>] [width: <pixels>] [height: <pixels>]`

**Supported formats:**
• MP4 (recommended)
• WebM
• MOV
• AVI
• MKV
• M4V

**Requirements:**
• Direct link to video file
• HTTPS/HTTP protocol
• Publicly accessible URL
• Video file extension

**Examples:**
```
/video url: https://example.com/myvideo.mp4
/video url: https://example.com/video.mp4 title: My Cool Video
/video url: https://example.com/video.mp4 width: 1280 height: 720
```

**Tips:**
• Use descriptive titles for better organization
• Standard video dimensions work best (16:9 ratio)
• Embed files are automatically cleaned up after 24 hours
• The original video must remain accessible for the embed to work

Use `/cleanup_embeds` to manually clean old embed files (Admin only)"#;

    ctx.say(help_text).await?;
    Ok(())
}

/// Manually clean up old embed files (Admin only)
#[poise::command(
    slash_command,
    default_member_permissions = "ADMINISTRATOR",
    guild_only = true
)]
pub async fn cleanup_embeds(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let embed_config = &ctx.data().config.web.embed;

    // Check if cleanup is disabled
    if embed_config.max_age_hours == 0 {
        ctx.say("ℹ️ cleanup is disabled in the configuration (max_age_hours = 0)").await?;
        return Ok(());
    }

    match cleanup_old_embeds(&embed_config.directory, embed_config.max_age_hours).await {
        Ok(cleaned_count) => {
            ctx.say(format!("🧹 cleanup complete: successfully cleaned up {} old embed files (max age: {} hours)",
                cleaned_count, embed_config.max_age_hours)).await?;
            info!("Manual cleanup completed: {} files removed", cleaned_count);
        }
        Err(e) => {
            error!("Manual cleanup failed: {}", e);
            ctx.say(format!("❌ cleanup failed: {}", e)).await?;
        }
    }

    Ok(())
}
