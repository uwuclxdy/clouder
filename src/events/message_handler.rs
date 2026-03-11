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

    // before touching the original message we need to make sure the bot has
    // both `manage messages` (to delete) and `manage webhooks` (to repost)
    // otherwise the normal flow will delete the message and then fail while
    // creating the webhook, which ends up with the user seeing a warning in
    // the logs and no uwufied text.  instead we send an informative embed and
    // bail out.
    {
        // fetch the guild channel object so we can inspect permissions
        let channel = match ctx.http.get_channel(message.channel_id).await {
            Ok(chan) => chan,
            Err(e) => {
                warn!("fetch channel before uwufy: {}", e);
                return;
            }
        };
        let guild_channel = match channel.guild() {
            Some(gc) => gc,
            None => {
                // not a guild channel, nothing to do
                return;
            }
        };

        // we only need the bot's id for the permissions call; the cached
        // current user is always available.
        let bot_id = ctx.cache.current_user().id;
        #[allow(deprecated)]
        if let Ok(perms) = guild_channel.permissions_for_user(&ctx.cache, bot_id) {
            if let Some(desc) = permission_error_message(perms) {
                // send an embed explaining which perms are missing
                let color = clouder_core::utils::get_embed_color(data, Some(guild_id.into())).await;
                // build a message manually instead of wrestling with closure
                let embed = serenity::CreateEmbed::new()
                    .color(color)
                    .title("missing perms")
                    .description(format!(
                        "{} \n> uwufy is **enabled** for <@{}>,",
                        desc, &user_id_str
                    ));
                let builder = serenity::CreateMessage::new().embed(embed);
                let _ = message.channel_id.send_message(&ctx.http, builder).await;
                return;
            }
        } else {
            // unable to compute permissions; be conservative and bail
            return;
        }
    }

    let mut uwufier = uwurs::UwUifier::new();
    uwufier.set_stutter_probability(0.5);
    uwufier.set_emoji_probability(0.25);
    let uwufied = match uwufier.uwuify(&message.content) {
        Ok(text) => text,
        Err(e) => {
            warn!("uwuify failed: {:?}", e);
            return;
        }
    };

    let webhooks = ctx
        .http
        .get_channel_webhooks(message.channel_id)
        .await
        .unwrap_or_default();

    let existing = webhooks
        .iter()
        .find(|w| w.name.as_deref() == Some("clouder"));

    let create_webhook = || async {
        message
            .channel_id
            .create_webhook(&ctx.http, serenity::CreateWebhook::new("clouder"))
            .await
    };

    let webhook = match existing {
        Some(wh) if wh.token.is_some() => wh.clone(),
        Some(wh) => {
            let _ = wh.delete(&ctx.http).await;
            match create_webhook().await {
                Ok(wh) => wh,
                Err(e) => {
                    warn!("recreate webhook: {}", e);
                    return;
                }
            }
        }
        None => match create_webhook().await {
            Ok(wh) => wh,
            Err(e) => {
                warn!("create webhook: {}", e);
                return;
            }
        },
    };

    if let Err(e) = message.delete(&ctx.http).await {
        warn!("delete message for uwufy: {}", e);
        return;
    }

    let execute = serenity::ExecuteWebhook::new()
        .content(&uwufied)
        .username(&message.author.name)
        .avatar_url(message.author.face());

    if let Err(e) = webhook.execute(&ctx.http, false, execute).await {
        warn!("execute uwufy webhook: {}", e);
        let fallback = serenity::CreateMessage::new()
            .content(format!("**{}:** {}", &message.author.name, &uwufied));
        let _ = message.channel_id.send_message(&ctx.http, fallback).await;
    }
}

/// returns an error description when the bot lacks permissions required for
/// uwufying.  we need both manage messages (delete) and manage webhooks
/// (repost); whenever either is missing we build a user-friendly sentence.
fn permission_error_message(perms: serenity::Permissions) -> Option<String> {
    // if the bot has administrator privileges we treat it as having every
    // permission; the helper consolidates the logic and makes unit testing
    // straightforward.
    let mut missing = Vec::new();
    if !clouder_core::utils::has_permission(perms, serenity::Permissions::MANAGE_MESSAGES) {
        missing.push("manage messages");
    }
    if !clouder_core::utils::has_permission(perms, serenity::Permissions::MANAGE_WEBHOOKS) {
        missing.push("manage webhooks");
    }
    if missing.is_empty() {
        None
    } else {
        Some(format!(
            "i need these to uwufy messages: `{}`",
            missing.join(", ")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use poise::serenity_prelude::Permissions;

    #[test]
    fn permission_message_all_missing() {
        let perms = Permissions::empty();
        let msg = permission_error_message(perms).unwrap();
        assert!(msg.contains("manage messages"));
        assert!(msg.contains("manage webhooks"));
    }

    #[test]
    fn permission_message_one_missing() {
        let perms = Permissions::MANAGE_MESSAGES;
        assert_eq!(
            permission_error_message(perms).unwrap(),
            "i need these to uwufy messages: `manage webhooks`"
        );
        let perms2 = Permissions::MANAGE_WEBHOOKS;
        assert_eq!(
            permission_error_message(perms2).unwrap(),
            "i need these to uwufy messages: `manage messages`"
        );
    }

    #[test]
    fn permission_message_admin_override() {
        // an administrator permission should satisfy both requirements
        let perms = Permissions::ADMINISTRATOR;
        assert!(permission_error_message(perms).is_none());

        // even if one of the checks would otherwise fail, administrator wins
        let perms2 = Permissions::ADMINISTRATOR | Permissions::empty();
        assert!(permission_error_message(perms2).is_none());
    }

    #[test]
    fn permission_message_none_missing() {
        let perms = Permissions::MANAGE_MESSAGES | Permissions::MANAGE_WEBHOOKS;
        assert!(permission_error_message(perms).is_none());
    }
}
