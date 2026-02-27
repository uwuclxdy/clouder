use anyhow::Result;
use clouder_core::config::AppState;
use clouder_core::database::uwufy::UwufyToggle;
use clouder_core::utils::get_default_embed_color;
use poise::serenity_prelude as serenity;
use serenity::{CreateEmbed, Mentionable};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, AppState, Error>;

#[poise::command(
    slash_command,
    required_permissions = "MANAGE_GUILD",
    guild_only,
    ephemeral
)]
pub async fn uwufy(
    ctx: Context<'_>,
    #[description = "user to toggle uwufy for"] user: serenity::User,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().to_string();
    let user_id = user.id.to_string();

    let enabled = UwufyToggle::toggle(&ctx.data().db, &guild_id, &user_id).await?;

    let status = if enabled { "enabled" } else { "disabled" };

    let embed = CreateEmbed::new()
        .title("uwufy toggled")
        .description(format!(
            "uwufy is now **{}** for {}",
            status,
            user.mention()
        ))
        .color(get_default_embed_color(ctx.data()));

    ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
        .await?;

    Ok(())
}
