use crate::config::AppState;
use crate::utils::get_default_embed_color;
use poise::serenity_prelude as serenity;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, AppState, Error>;

#[poise::command(slash_command, guild_only)]
pub async fn selfroles(ctx: Context<'_>) -> Result<(), Error> {
    let base_url = &ctx.data().config.web.base_url;
    let guild_id = ctx.guild_id().unwrap().to_string();

    let embed = serenity::CreateEmbed::new()
        .title("self-roles config")
        .description("click the link below to configure self-roles for your server")
        .field(
            "web dashboard",
            format!("{}/dashboard/{}/selfroles", base_url, guild_id),
            false,
        )
        .color(get_default_embed_color(ctx.data()))
        .footer(serenity::CreateEmbedFooter::new("you need 'manage roles' permission to configure self-roles"));

    ctx.send(
        poise::CreateReply::default()
            .embed(embed)
            .ephemeral(true)
    ).await?;

    Ok(())
}
