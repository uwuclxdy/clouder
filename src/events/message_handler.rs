use crate::logging::warn;
use clouder_core::config::AppState;
use clouder_core::database::uwufy::UwufyToggle;
use poise::serenity_prelude as serenity;

pub async fn handle_uwufy_message(
    ctx: &serenity::Context,
    message: &serenity::Message,
    data: &AppState,
) {
    if message.author.bot {
        return;
    }

    let Some(guild_id) = message.guild_id else {
        return;
    };

    if message.content.is_empty() {
        return;
    }

    if !message.attachments.is_empty() {
        return;
    }

    let guild_id_str = guild_id.to_string();
    let user_id_str = message.author.id.to_string();

    match UwufyToggle::is_enabled(&data.db, &guild_id_str, &user_id_str).await {
        Ok(true) => {}
        _ => return,
    }

    let mut uwufier = uwurs::UwUifier::new();
    uwufier.set_stutter_probability(0.5);
    uwufier.set_emoji_probability(0.25);
    let uwufied = uwufier.uwuify(&message.content).unwrap();

    if let Err(e) = message.delete(&ctx.http).await {
        warn!("delete message for uwufy: {}", e);
        return;
    }

    let webhooks = ctx
        .http
        .get_channel_webhooks(message.channel_id)
        .await
        .unwrap_or_default();

    let webhook = if let Some(wh) = webhooks
        .iter()
        .find(|w| w.name.as_deref() == Some("clouder"))
    {
        wh.clone()
    } else {
        match message
            .channel_id
            .create_webhook(&ctx.http, serenity::CreateWebhook::new("clouder"))
            .await
        {
            Ok(wh) => wh,
            Err(e) => {
                warn!("create webhook: {}", e);
                return;
            }
        }
    };

    let execute = serenity::ExecuteWebhook::new()
        .content(&uwufied)
        .username(&message.author.name)
        .avatar_url(message.author.face());

    if let Err(e) = webhook.execute(&ctx.http, false, execute).await {
        warn!("execute uwufy webhook: {}", e);
    }
}
