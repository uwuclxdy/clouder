use crate::config::AppState;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, AppState, Error>;

#[poise::command(slash_command)]
pub async fn video(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Video command placeholder - not implemented yet").await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn video_help(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Video help command placeholder - not implemented yet").await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn cleanup_embeds(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Cleanup embeds command placeholder - not implemented yet").await?;
    Ok(())
}