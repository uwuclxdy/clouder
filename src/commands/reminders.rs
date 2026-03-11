use anyhow::Result;
use clouder_core::config::AppState;
use clouder_core::database::reminders::{ReminderConfig, ReminderType, UserSettings};
use clouder_core::utils::get_embed_color;
use poise::serenity_prelude as serenity;
use serenity::CreateEmbed;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, AppState, Error>;

/// view active reminders for this server or your subscribed reminders in DMs
#[poise::command(slash_command)]
pub async fn reminders(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();
    let color = get_embed_color(data, ctx.guild_id().map(|g| g.get())).await;

    if let Some(guild_id) = ctx.guild_id() {
        // in server: show all server reminders
        let configs = ReminderConfig::get_by_guild(&data.db, &guild_id.to_string())
            .await
            .unwrap_or_default();

        if configs.is_empty() {
            let embed = CreateEmbed::new()
                .title("reminders")
                .description(
                    "no reminders configured for this server\nconfigure them at the dashboard :3",
                )
                .color(color);
            ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
                .await?;
            return Ok(());
        }

        let mut lines = Vec::new();
        for cfg in &configs {
            let type_label = match cfg.reminder_type {
                ReminderType::Wysi => "wysi (7:27)",
                ReminderType::FemboyFriday => "femboy friday",
                ReminderType::Custom => "custom",
            };
            let status = if cfg.enabled { "✓" } else { "✗" };
            let channel = cfg
                .channel_id
                .as_deref()
                .map(|c| format!("<#{}>", c))
                .unwrap_or_else(|| "no channel".to_string());
            let tz = &cfg.timezone;
            let schedule = match cfg.reminder_type {
                ReminderType::Wysi => format!(
                    "{} AM / {} PM",
                    cfg.wysi_morning_time.as_deref().unwrap_or("07:27"),
                    cfg.wysi_evening_time.as_deref().unwrap_or("19:27")
                ),
                ReminderType::FemboyFriday => format!(
                    "fridays @ {}",
                    cfg.femboy_friday_time.as_deref().unwrap_or("00:00")
                ),
                ReminderType::Custom => "custom schedule".to_string(),
            };
            lines.push(format!(
                "{} **{}** — {} — {} ({})",
                status, type_label, channel, schedule, tz
            ));
        }

        let embed = CreateEmbed::new()
            .title("server reminders")
            .description(lines.join("\n"))
            .color(color)
            .footer(serenity::CreateEmbedFooter::new(
                "configure at the dashboard",
            ));

        ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
            .await?;
    } else {
        // in DMs: show user's subscribed reminders + timezone
        let user_id = ctx.author().id.to_string();
        let settings = UserSettings::get(&data.db, &user_id).await.unwrap_or(None);

        let tz = settings
            .as_ref()
            .map(|s| s.timezone.as_str())
            .unwrap_or("UTC");
        let dm_enabled = settings
            .as_ref()
            .map(|s| s.dm_reminders_enabled)
            .unwrap_or(true);

        let subscriptions = clouder_core::database::reminders::ReminderSubscription::get_by_user(
            &data.db, &user_id,
        )
        .await
        .unwrap_or_default();

        let dm_status = if dm_enabled { "enabled" } else { "disabled" };
        let sub_count = subscriptions.len();

        let desc = if sub_count == 0 {
            format!(
                "**dm reminders:** {}\n**timezone:** {}\n\nno active subscriptions\nsubscribe to reminders from the web dashboard :3",
                dm_status, tz
            )
        } else {
            format!(
                "**dm reminders:** {}\n**timezone:** {}\n**subscriptions:** {}",
                dm_status, tz, sub_count
            )
        };

        let embed = CreateEmbed::new()
            .title("your reminders")
            .description(desc)
            .color(color)
            .footer(serenity::CreateEmbedFooter::new("manage at the dashboard"));

        ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
            .await?;
    }

    Ok(())
}
