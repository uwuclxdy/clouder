use crate::config::AppState;
use crate::utils::{get_bot_invite_url, get_guild_icon_url, get_user_avatar_url};
use crate::web::session_extractor::extract_session_data;
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{Html, Redirect},
};

/// Check if the bot is present in a guild
async fn is_bot_in_guild(state: &AppState, guild_id: &str) -> bool {
    if let Ok(guild_id_u64) = guild_id.parse::<u64>() {
        crate::web::get_bot_member_info(&state.http, guild_id_u64.into())
            .await
            .is_ok()
    } else {
        false
    }
}

pub async fn server_list(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Html<String>, Redirect> {
    let session = extract_session_data(&headers)
        .await
        .map_err(|_| Redirect::temporary("/auth/login"))?;

    let user = session
        .1
        .ok_or_else(|| Redirect::temporary("/auth/login"))?;
    let manageable_guilds = user.get_manageable_guilds();

    // Filter guilds to only show those where the bot is also present
    let mut bot_accessible_guilds = Vec::new();
    for guild in manageable_guilds {
        if is_bot_in_guild(&state, &guild.id).await {
            bot_accessible_guilds.push(guild);
        }
    }

    let mut guilds_html = String::new();
    for guild in &bot_accessible_guilds {
        let icon_url = guild
            .icon
            .as_ref()
            .map(|icon| format!("https://cdn.discordapp.com/icons/{}/{}.png", guild.id, icon))
            .unwrap_or_else(|| "https://cdn.discordapp.com/embed/avatars/0.png".to_string());

        let guild_card = include_str!("templates/partials/guild_card.html")
            .replace("{{GUILD_ID}}", &guild.id)
            .replace("{{ICON_URL}}", &icon_url)
            .replace("{{GUILD_NAME}}", &guild.name)
            .replace(
                "{{PERMISSION_TEXT}}",
                if guild.owner { "Owner" } else { "Manage Roles" },
            );

        guilds_html.push_str(&guild_card);
    }

    if guilds_html.is_empty() {
        let manageable_count = user.get_manageable_guilds().len();
        if manageable_count > 0 {
            // User has manageable guilds, but bot is not in any of them
            guilds_html = r#"<div class="no-servers">
                <h3>Bot Not Added to Your Servers</h3>
                <p>You have servers with the required permissions, but the bot hasn't been added to them yet.</p>
                <p>Click "Add to Server" above to invite the bot to your servers.</p>
            </div>"#.to_string();
        } else if !user.guilds.is_empty() {
            // User has guilds but no manage permissions
            guilds_html = include_str!("templates/partials/no_manageable_servers.html").to_string();
        } else {
            // User has no guilds at all
            guilds_html = include_str!("templates/partials/guild_load_error.html").to_string();
        }
    }

    let user_avatar = user
        .user
        .avatar
        .as_ref()
        .map(|avatar| {
            format!(
                "https://cdn.discordapp.com/avatars/{}/{}.png",
                user.user.id, avatar
            )
        })
        .unwrap_or_else(|| "https://cdn.discordapp.com/embed/avatars/0.png".to_string());

    // Generate Discord invite URL with full permissions for Clouder bot
    let invite_url = format!(
        "https://discord.com/oauth2/authorize?client_id={}&permissions=268697088&response_type=code&redirect_uri={}&integration_type=0&scope=bot",
        state.config.web.oauth.client_id, state.config.web.oauth.redirect_uri
    );

    let template = include_str!("templates/server_list.html")
        .replace("{{COMMON_CSS}}", include_str!("static/css/common.css"))
        .replace(
            "{{DASHBOARD_CSS}}",
            include_str!("static/css/dashboard.css"),
        )
        .replace("{{USER_AVATAR}}", &user_avatar)
        .replace("{{USER_NAME}}", &user.user.username)
        .replace("{{INVITE_URL}}", &invite_url)
        .replace("{{GUILDS_HTML}}", &guilds_html);

    Ok(Html(template))
}

pub async fn user_settings(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Html<String>, Redirect> {
    let session = extract_session_data(&headers)
        .await
        .map_err(|_| Redirect::temporary("/auth/login"))?;

    let user = session
        .1
        .ok_or_else(|| Redirect::temporary("/auth/login"))?;

    let user_avatar = get_user_avatar_url(&user.user.id, user.user.avatar.as_ref());

    // Generate Discord invite URL with full permissions for Clouder bot
    let invite_url = get_bot_invite_url(
        &state.config.web.oauth.client_id,
        Some(&state.config.web.oauth.redirect_uri),
    );

    // TODO: Load actual user settings from database
    let current_timezone = "UTC";
    let dm_reminders_enabled = true;
    let dm_reminders_checked = if dm_reminders_enabled { "checked" } else { "" };

    let template = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <title>User Settings - Clouder Bot</title>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>
        {{COMMON_CSS}}
        {{DASHBOARD_CSS}}
    </style>
</head>
<body>
    <nav class="navbar">
        <div class="navbar-content">
            <div class="navbar-left">
                <div class="logo">clouder</div>
                <div class="breadcrumb">
                    <a href="/" class="breadcrumb-item">Dashboard</a>
                    <span class="breadcrumb-separator">/</span>
                    <span class="breadcrumb-item current">Settings</span>
                </div>
            </div>
            <div class="navbar-right">
                <a href="{{INVITE_URL}}" target="_blank" class="add-server-btn">Add to Server</a>
                <a href="/auth/logout" class="logout-btn">Logout</a>
            </div>
        </div>
    </nav>
    <div class="container">
        <div class="user-info">
            <div class="user-info-left">
                <img src="{{USER_AVATAR}}" alt="{{USER_NAME}}" class="user-avatar">
                <div>
                    <h3 class="welcome-username">{{USER_NAME}}</h3>
                    <p class="welcome-subtitle">Manage your personal settings and preferences</p>
                </div>
            </div>
        </div>

        <div class="settings-content">
            <div class="settings-section">
                <h2>Personal Preferences</h2>

                <div class="setting-item">
                    <div class="setting-header">
                        <h3>Timezone</h3>
                        <p>Set your timezone for DM reminder delivery times</p>
                    </div>
                    <div class="setting-control">
                        <select id="timezone" name="timezone" class="settings-select">
                            <option value="">Loading timezones...</option>
                        </select>
                    </div>
                </div>

                <div class="setting-item">
                    <div class="setting-header">
                        <h3>DM Reminders</h3>
                        <p>Enable or disable reminder notifications via direct messages</p>
                    </div>
                    <div class="setting-control">
                        <label class="toggle-switch">
                            <input type="checkbox" id="dmReminders" name="dm_reminders_enabled" {{DM_REMINDERS_CHECKED}} onchange="toggleRemindersCard()">
                            <span class="toggle-slider"></span>
                        </label>
                    </div>
                </div>

                <div class="setting-item">
                    <button id="saveSettings" class="btn primary-btn">Save Settings</button>
                </div>
            </div>

            <div class="settings-section reminders-card" id="remindersCard">
                <h2>Reminder Management</h2>

                <div class="reminders-notice">
                    <p><strong>Coming Soon</strong></p>
                    <p>Reminder management features are currently under development. This section will show your subscribed reminders and allow you to manage them.</p>
                </div>
            </div>
        </div>
    </div>

    <script>
        {{COMMON_JS}}

        const userId = '{{USER_ID}}';
        let currentSettings = {
            timezone: '{{CURRENT_TIMEZONE}}',
            dm_reminders_enabled: {{DM_REMINDERS_ENABLED}}
        };

        // Common timezone list with GMT offsets
        const timezones = [
            { value: 'UTC-12', label: 'UTC−12:00' },
            { value: 'UTC-11', label: 'UTC−11:00' },
            { value: 'UTC-10', label: 'UTC−10:00' },
            { value: 'UTC-9', label: 'UTC−09:00' },
            { value: 'UTC-8', label: 'UTC−08:00' },
            { value: 'UTC-7', label: 'UTC−07:00' },
            { value: 'UTC-6', label: 'UTC−06:00' },
            { value: 'UTC-5', label: 'UTC−05:00' },
            { value: 'UTC-4', label: 'UTC−04:00' },
            { value: 'UTC-3', label: 'UTC−03:00' },
            { value: 'UTC-2', label: 'UTC−02:00' },
            { value: 'UTC-1', label: 'UTC−01:00' },
            { value: 'UTC', label: 'UTC±00:00' },
            { value: 'UTC+1', label: 'UTC+01:00' },
            { value: 'UTC+2', label: 'UTC+02:00' },
            { value: 'UTC+3', label: 'UTC+03:00' },
            { value: 'UTC+4', label: 'UTC+04:00' },
            { value: 'UTC+5', label: 'UTC+05:00' },
            { value: 'UTC+6', label: 'UTC+06:00' },
            { value: 'UTC+7', label: 'UTC+07:00' },
            { value: 'UTC+8', label: 'UTC+08:00' },
            { value: 'UTC+9', label: 'UTC+09:00' },
            { value: 'UTC+10', label: 'UTC+10:00' },
            { value: 'UTC+11', label: 'UTC+11:00' },
            { value: 'UTC+12', label: 'UTC+12:00' },
            { value: 'UTC+13', label: 'UTC+13:00' },
            { value: 'UTC+14', label: 'UTC+14:00' }
        ];

        function populateTimezones() {
            const timezoneSelect = document.getElementById('timezone');
            timezoneSelect.innerHTML = '';

            timezones.forEach(tz => {
                const option = document.createElement('option');
                option.value = tz.value;
                option.textContent = tz.label;
                if (tz.value === currentSettings.timezone) {
                    option.selected = true;
                }
                timezoneSelect.appendChild(option);
            });
        }

        async function saveSettings() {
            const saveBtn = document.getElementById('saveSettings');
            const originalText = saveBtn.textContent;

            setButtonLoading(saveBtn, true, 'Saving...');

            try {
                const timezone = document.getElementById('timezone').value;
                const dmReminders = document.getElementById('dmReminders').checked;

                const response = await fetch('/api/user/settings', {
                    method: 'PUT',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify({
                        timezone: timezone,
                        dm_reminders_enabled: dmReminders
                    })
                });

                const data = await response.json();

                if (data.success) {
                    currentSettings = { timezone, dm_reminders_enabled: dmReminders };
                    showMessage('Settings saved successfully!', 'success');
                } else {
                    throw new Error(data.message || 'Failed to save settings');
                }
            } catch (error) {
                console.error('Failed to save settings:', error);
                showMessage(error.message || 'Failed to save settings. Please try again.', 'error');
            } finally {
                setButtonLoading(saveBtn, false, originalText);
            }
        }

        function toggleRemindersCard() {
            const remindersCard = document.getElementById('remindersCard');
            const dmRemindersToggle = document.getElementById('dmReminders');

            if (dmRemindersToggle.checked) {
                remindersCard.classList.remove('disabled');
            } else {
                remindersCard.classList.add('disabled');
            }
        }

        document.addEventListener('DOMContentLoaded', function() {
            populateTimezones();
            toggleRemindersCard(); // Set initial state

            document.getElementById('saveSettings').addEventListener('click', saveSettings);
        });
    </script>
</body>
</html>"#
        .replace("{{COMMON_CSS}}", include_str!("static/css/common.css"))
        .replace("{{DASHBOARD_CSS}}", include_str!("static/css/dashboard.css"))
        .replace("{{COMMON_JS}}", include_str!("static/js/common.js"))
        .replace("{{USER_AVATAR}}", &user_avatar)
        .replace("{{USER_NAME}}", &user.user.username)
        .replace("{{USER_ID}}", &user.user.id)
        .replace("{{CURRENT_TIMEZONE}}", current_timezone)
        .replace("{{DM_REMINDERS_ENABLED}}", &dm_reminders_enabled.to_string())
        .replace("{{DM_REMINDERS_CHECKED}}", dm_reminders_checked)
        .replace("{{INVITE_URL}}", &invite_url);

    Ok(Html(template))
}

pub async fn guild_dashboard(
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    State(_state): State<AppState>,
) -> Result<Html<String>, Redirect> {
    let session = extract_session_data(&headers)
        .await
        .map_err(|_| Redirect::temporary("/auth/login"))?;

    let user = session
        .1
        .ok_or_else(|| Redirect::temporary("/auth/login"))?;

    if !user.has_manage_roles_in_guild(&guild_id) && !user.has_administrator_in_guild(&guild_id) {
        return Err(Redirect::temporary("/"));
    }

    let guild = user.guilds.iter().find(|g| g.id == guild_id).unwrap();

    let guild_icon = get_guild_icon_url(&guild.id, guild.icon.as_ref());

    // Generate Discord invite URL with full permissions for Clouder bot
    let invite_url = get_bot_invite_url(&_state.config.web.oauth.client_id, None);

    let template = include_str!("templates/guild_dashboard.html")
        .replace("{{COMMON_CSS}}", include_str!("static/css/common.css"))
        .replace(
            "{{DASHBOARD_CSS}}",
            include_str!("static/css/dashboard.css"),
        )
        .replace("{{GUILD_NAME}}", &guild.name)
        .replace("{{GUILD_ICON}}", &guild_icon)
        .replace("{{GUILD_ID}}", &guild_id)
        .replace("{{INVITE_URL}}", &invite_url);

    Ok(Html(template))
}

pub async fn selfroles_list(
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    State(_state): State<AppState>,
) -> Result<Html<String>, Redirect> {
    let session = extract_session_data(&headers)
        .await
        .map_err(|_| Redirect::temporary("/auth/login"))?;

    let user = session
        .1
        .ok_or_else(|| Redirect::temporary("/auth/login"))?;

    if !user.has_manage_roles_in_guild(&guild_id) && !user.has_administrator_in_guild(&guild_id) {
        return Err(Redirect::temporary("/"));
    }

    let guild = user.guilds.iter().find(|g| g.id == guild_id).unwrap();

    // Generate Discord invite URL with full permissions for Clouder bot
    let invite_url = get_bot_invite_url(&_state.config.web.oauth.client_id, None);

    let template = include_str!("templates/selfroles_list.html")
        .replace("{{COMMON_CSS}}", include_str!("static/css/common.css"))
        .replace(
            "{{DASHBOARD_CSS}}",
            include_str!("static/css/dashboard.css"),
        )
        .replace("{{COMMON_JS}}", include_str!("static/js/common.js"))
        .replace("{{SELFROLES_JS}}", include_str!("static/js/selfroles.js"))
        .replace("{{GUILD_NAME}}", &guild.name)
        .replace("{{GUILD_ID}}", &guild_id)
        .replace("{{INVITE_URL}}", &invite_url);

    Ok(Html(template))
}

pub async fn selfroles_create(
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Html<String>, Redirect> {
    let session = extract_session_data(&headers)
        .await
        .map_err(|_| Redirect::temporary("/auth/login"))?;

    let user = session
        .1
        .ok_or_else(|| Redirect::temporary("/auth/login"))?;

    if !user.has_manage_roles_in_guild(&guild_id) && !user.has_administrator_in_guild(&guild_id) {
        return Err(Redirect::temporary("/"));
    }

    let guild = user.guilds.iter().find(|g| g.id == guild_id).unwrap();
    render_selfroles_form(&guild_id, &guild.name, None, &state)
}

pub async fn selfroles_edit(
    Path((guild_id, config_id)): Path<(String, String)>,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Html<String>, Redirect> {
    let session = extract_session_data(&headers)
        .await
        .map_err(|_| Redirect::temporary("/auth/login"))?;

    let user = session
        .1
        .ok_or_else(|| Redirect::temporary("/auth/login"))?;

    if !user.has_manage_roles_in_guild(&guild_id) && !user.has_administrator_in_guild(&guild_id) {
        return Err(Redirect::temporary("/"));
    }

    let guild = user.guilds.iter().find(|g| g.id == guild_id).unwrap();
    render_selfroles_form(&guild_id, &guild.name, Some(&config_id), &state)
}

fn render_selfroles_form(
    guild_id: &str,
    guild_name: &str,
    config_id: Option<&str>,
    state: &AppState,
) -> Result<Html<String>, Redirect> {
    let (page_title, header_title, header_description, breadcrumb_current, button_text) =
        if config_id.is_some() {
            (
                "Edit Self-Role Message",
                "Edit Self-Role Message",
                "Edit interactive role assignment message for",
                "Edit",
                "Update Self-Role Message",
            )
        } else {
            (
                "Create Self-Role Message",
                "Create Self-Role Message",
                "Create a new interactive role assignment message for",
                "Create",
                "Create Self-Role Message",
            )
        };

    // Generate Discord invite URL with full permissions for Clouder bot
    let invite_url = get_bot_invite_url(&state.config.web.oauth.client_id, None);

    let template = include_str!("templates/selfroles_form.html")
        .replace("{{COMMON_CSS}}", include_str!("static/css/common.css"))
        .replace(
            "{{DASHBOARD_CSS}}",
            include_str!("static/css/dashboard.css"),
        )
        .replace("{{COMMON_JS}}", include_str!("static/js/common.js"))
        .replace(
            "{{SELFROLES_CONFIG_JS}}",
            include_str!("static/js/selfroles_config.js"),
        )
        .replace("{{SELFROLES_JS}}", include_str!("static/js/selfroles.js"))
        .replace("{{GUILD_NAME}}", guild_name)
        .replace("{{GUILD_ID}}", guild_id)
        .replace("{{PAGE_TITLE}}", page_title)
        .replace("{{HEADER_TITLE}}", header_title)
        .replace("{{HEADER_DESCRIPTION}}", header_description)
        .replace("{{BREADCRUMB_CURRENT}}", breadcrumb_current)
        .replace("{{BUTTON_TEXT}}", button_text)
        .replace("{{INVITE_URL}}", &invite_url);

    Ok(Html(template))
}

pub async fn feature_request(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Html<String>, Redirect> {
    let session = extract_session_data(&headers)
        .await
        .map_err(|_| Redirect::temporary("/auth/login"))?;

    let user = session
        .1
        .ok_or_else(|| Redirect::temporary("/auth/login"))?;

    let user_avatar = get_user_avatar_url(&user.user.id, user.user.avatar.as_ref());

    // Generate Discord invite URL with full permissions for Clouder bot
    let invite_url = get_bot_invite_url(
        &state.config.web.oauth.client_id,
        Some(&state.config.web.oauth.redirect_uri),
    );

    let template = include_str!("templates/feature_request.html")
        .replace("{{COMMON_CSS}}", include_str!("static/css/common.css"))
        .replace(
            "{{DASHBOARD_CSS}}",
            include_str!("static/css/dashboard.css"),
        )
        .replace("{{COMMON_JS}}", include_str!("static/js/common.js"))
        .replace("{{USER_AVATAR}}", &user_avatar)
        .replace("{{USER_NAME}}", &user.user.username)
        .replace("{{USER_ID}}", &user.user.id.to_string())
        .replace("{{INVITE_URL}}", &invite_url);

    Ok(Html(template))
}
