use chrono::{Duration, NaiveTime, Utc};
use chrono_tz::Tz;
use clouder_core::{
    config::AppState,
    database::reminders::{
        ReminderConfig, ReminderLog, ReminderSubscription, ReminderType, UserSettings,
    },
};
use serde_json::json;
use serenity::all::{ChannelId, CreateMessage};
use tokio::time::{Duration as TokioDuration, interval};
use tracing::{error, info, warn};

/// Start the background reminder scheduler (runs every 60 seconds)
pub fn start_scheduler(state: AppState) {
    tokio::spawn(async move {
        let mut ticker = interval(TokioDuration::from_secs(60));
        loop {
            ticker.tick().await;
            if let Err(e) = run_due_reminders(&state).await {
                error!("scheduler tick error: {}", e);
            }
        }
    });
}

async fn run_due_reminders(state: &AppState) -> anyhow::Result<()> {
    let now_utc = Utc::now();

    // fetch all enabled reminders
    let rows = sqlx::query_as::<_, (i64, String, String, Option<String>, Option<String>, String)>(
        "SELECT id, guild_id, reminder_type, wysi_morning_time, wysi_evening_time, timezone
         FROM reminder_configs
         WHERE enabled = 1 AND channel_id IS NOT NULL",
    )
    .fetch_all(state.db.as_ref())
    .await?;

    for (id, guild_id, reminder_type_str, morning, evening, tz_str) in rows {
        let Some(rtype) = ReminderType::parse(&reminder_type_str) else {
            continue;
        };

        let tz: Tz = match tz_str.parse() {
            Ok(t) => t,
            Err(_) => {
                warn!("invalid timezone '{}' for reminder {}", tz_str, id);
                continue;
            }
        };

        let now_local = now_utc.with_timezone(&tz);

        let should_fire = match rtype {
            ReminderType::Wysi => {
                let morning_time = parse_hhmm(morning.as_deref().unwrap_or("07:27"))
                    .unwrap_or(NaiveTime::from_hms_opt(7, 27, 0).unwrap());
                let evening_time = parse_hhmm(evening.as_deref().unwrap_or("19:27"))
                    .unwrap_or(NaiveTime::from_hms_opt(19, 27, 0).unwrap());

                let current = now_local.time();
                // fire if within the current minute window
                is_within_minute(current, morning_time) || is_within_minute(current, evening_time)
            }
            ReminderType::Custom => false,
        };

        if !should_fire {
            continue;
        }

        // debounce: skip if we already fired this reminder within the past 55 seconds
        if already_fired_recently(state, id, 55).await {
            continue;
        }

        // fetch full config for message content
        let config = match ReminderConfig::get_by_id(&state.db, id)
            .await
            .ok()
            .flatten()
        {
            Some(c) => c,
            None => continue,
        };

        let channel_id: u64 = match config.channel_id.as_deref().and_then(|s| s.parse().ok()) {
            Some(c) => c,
            None => continue,
        };

        let ping_roles = clouder_core::database::reminders::ReminderPingRole::get_by_config(
            &state.db, config.id,
        )
        .await
        .unwrap_or_default();

        let role_mentions: String = ping_roles
            .iter()
            .map(|r| format!("<@&{}>", r.role_id))
            .collect::<Vec<_>>()
            .join(" ");

        // build countdown for WYSI
        let next_727 = if rtype == ReminderType::Wysi {
            next_727_timestamp(&tz)
        } else {
            None
        };

        let msg = build_reminder_message(&config, &rtype, &role_mentions, next_727.as_deref());

        let mut channel_sent = false;
        let send_result = state
            .http
            .send_message(ChannelId::new(channel_id), vec![], &msg)
            .await;

        match send_result {
            Ok(_) => {
                channel_sent = true;
                info!("reminder {} fired in guild {}", id, guild_id);
            }
            Err(e) => {
                error!("reminder {} send failed: {}", id, e);
            }
        }

        // DM subscribers
        let (dm_count, dm_failed) = send_dms(state, &config, &rtype).await;

        let status = if channel_sent { "success" } else { "error" };
        let err_msg = if !channel_sent {
            Some("failed to send to channel")
        } else {
            None
        };

        let _ = ReminderLog::create(
            &state.db,
            id,
            status,
            err_msg,
            channel_sent,
            dm_count as i64,
            dm_failed as i64,
        )
        .await;
    }

    Ok(())
}

async fn send_dms(
    state: &AppState,
    config: &ReminderConfig,
    rtype: &ReminderType,
) -> (usize, usize) {
    let subs = match ReminderSubscription::get_by_config(&state.db, config.id).await {
        Ok(s) => s,
        Err(_) => return (0, 0),
    };

    let mut sent = 0;
    let mut failed = 0;

    for sub in &subs {
        // check user has DMs enabled
        let settings = UserSettings::get(&state.db, &sub.user_id)
            .await
            .unwrap_or(None);
        if settings.as_ref().is_some_and(|s| !s.dm_reminders_enabled) {
            continue;
        }

        let user_tz = settings
            .as_ref()
            .and_then(|s| s.timezone.parse::<Tz>().ok());

        let next_str: Option<String> = match rtype {
            ReminderType::Wysi => user_tz.as_ref().and_then(next_727_timestamp),
            _ => None,
        };

        let msg = build_reminder_message(config, rtype, "", next_str.as_deref());

        let user_id_u64: u64 = match sub.user_id.parse() {
            Ok(u) => u,
            Err(_) => {
                failed += 1;
                continue;
            }
        };

        let send = async {
            let channel = state
                .http
                .create_private_channel(&json!({ "recipient_id": user_id_u64 }))
                .await?;
            state.http.send_message(channel.id, vec![], &msg).await?;
            Ok::<_, serenity::Error>(())
        };

        match send.await {
            Ok(_) => sent += 1,
            Err(e) => {
                warn!("dm failed for user {}: {}", sub.user_id, e);
                failed += 1;
                // unsubscribe on blocked DMs (can't open DM)
                if e.to_string().contains("Cannot send messages to this user") {
                    let _ =
                        ReminderSubscription::unsubscribe(&state.db, &sub.user_id, config.id).await;
                }
            }
        }
    }

    (sent, failed)
}

fn build_reminder_message(
    config: &ReminderConfig,
    rtype: &ReminderType,
    role_mentions: &str,
    next_727: Option<&str>,
) -> CreateMessage {
    let mut msg = CreateMessage::new();

    if !role_mentions.is_empty() {
        msg = msg.content(role_mentions);
    }

    if config.message_type == "embed" {
        use serenity::all::{CreateEmbed, CreateEmbedFooter};

        let default_desc = match rtype {
            ReminderType::Wysi => {
                let countdown = next_727
                    .map(|t| format!("\nnext: {}", t))
                    .unwrap_or_default();
                format!("it's 7:27! when you see it :3{}", countdown)
            }
            ReminderType::Custom => String::new(),
        };

        let title = config.embed_title.as_deref().unwrap_or(match rtype {
            ReminderType::Wysi => "7:27",
            ReminderType::Custom => "reminder",
        });

        let desc = config.embed_description.as_deref().unwrap_or(&default_desc);

        let color = config.embed_color.unwrap_or(0xFFFFFF) as u32;

        let embed = CreateEmbed::new()
            .title(title)
            .description(desc)
            .colour(color)
            .footer(CreateEmbedFooter::new("clouder"));

        msg = msg.embed(embed);
    } else {
        let default_content = match rtype {
            ReminderType::Wysi => {
                let countdown = next_727
                    .map(|t| format!(" (next: {})", t))
                    .unwrap_or_default();
                format!("it's 7:27! when you see it :3{}", countdown)
            }
            ReminderType::Custom => String::new(),
        };

        let content = config
            .message_content
            .as_deref()
            .unwrap_or(&default_content);

        let full = if role_mentions.is_empty() {
            content.to_string()
        } else {
            // role mentions already set as content; embed the message in the same call
            // for text mode we put everything together
            format!("{} {}", role_mentions, content)
        };

        msg = CreateMessage::new().content(full);
    }

    msg
}

pub fn parse_hhmm(s: &str) -> Option<NaiveTime> {
    let parts: Vec<&str> = s.splitn(2, ':').collect();
    if parts.len() != 2 {
        return None;
    }
    let h: u32 = parts[0].parse().ok()?;
    let m: u32 = parts[1].parse().ok()?;
    NaiveTime::from_hms_opt(h, m, 0)
}

fn is_within_minute(current: NaiveTime, target: NaiveTime) -> bool {
    let diff = current.signed_duration_since(target);
    diff >= Duration::zero() && diff < Duration::minutes(1)
}

pub fn next_727_timestamp(tz: &Tz) -> Option<String> {
    let now = Utc::now().with_timezone(tz);
    let current_time = now.time();
    let morning = NaiveTime::from_hms_opt(7, 27, 0)?;
    let evening = NaiveTime::from_hms_opt(19, 27, 0)?;

    let candidates = [morning, evening];
    let next = candidates
        .iter()
        .map(|&t| {
            let diff = t.signed_duration_since(current_time);
            if diff > Duration::zero() {
                diff
            } else {
                Duration::hours(24) + diff
            }
        })
        .min()?;

    let next_ts = now + next;
    Some(clouder_core::utils::discord_timestamp(
        next_ts.timestamp(),
        'R',
    ))
}

async fn already_fired_recently(state: &AppState, config_id: i64, within_secs: i64) -> bool {
    let result = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(*) FROM reminder_logs WHERE config_id = ? AND created_at >= datetime('now', ?)",
    )
    .bind(config_id)
    .bind(format!("-{} seconds", within_secs))
    .fetch_one(state.db.as_ref())
    .await;

    matches!(result, Ok((n,)) if n > 0)
}
