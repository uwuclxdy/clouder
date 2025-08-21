use crate::config::AppState;
use crate::utils::embed::{generate_embed_html, generate_embed_id, sanitize_html_content, save_embed_file};
use tracing::{error, info};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, AppState, Error>;

/// Create a video embed to bypasses Discord's 10MB limit
#[poise::command(
    slash_command,
    guild_only = true
)]
pub async fn video(
    ctx: Context<'_>,
    #[description = "Direct URL to the video file (e.g., https://example.com/video.mp4)"] url: String,
    #[description = "Custom title for the video (optional)"] title: Option<String>
) -> Result<(), Error> {
    ctx.defer().await?;

    let embed_id = generate_embed_id(12);
    let html_content = generate_embed_html(&url);

    // Save HTML file
    let embed_dir = &ctx.data().config.web.embed.directory;
    match save_embed_file(embed_dir, &embed_id, &html_content).await {
        Ok(_) => {
            let base_url = &ctx.data().config.web.base_url;
            let embed_url = format!("{}/video/{}.html", base_url, embed_id);

            let video_title = title.clone().unwrap_or_else(|| url.to_string());
            let sanitized_title = sanitize_html_content(&video_title);

            let mut url_message = embed_url.clone();
            if title.is_some() {
                url_message = format!("[fuc{sanitized_title}]({embed_url})");
            }

            ctx.say(url_message).await?;

            info!("Created video embed {} for URL: {}", embed_id, url);
        }
        Err(e) => {
            error!("Failed to save embed file: {}", e);
            ctx.send(poise::CreateReply::default()
                .content("❌ failed to create embed file. please try again later.")
                .ephemeral(true)).await?;
        }
    }

    Ok(())
}
