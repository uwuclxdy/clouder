use crate::config::AppState;
use crate::web::session_extractor::extract_session_data;
use axum::{
    extract::{Path, State},
    response::{Html, Redirect},
    http::HeaderMap,
};

pub async fn server_list(
    headers: HeaderMap,
    State(_state): State<AppState>,
) -> Result<Html<String>, Redirect> {
    let session = extract_session_data(&headers).await
        .map_err(|_| Redirect::temporary("/auth/login"))?;

    let user = session.1.ok_or_else(|| Redirect::temporary("/auth/login"))?;
    let manageable_guilds = user.get_manageable_guilds();

    let mut guilds_html = String::new();
    for guild in manageable_guilds {
        let icon_url = guild.icon.as_ref()
            .map(|icon| format!("https://cdn.discordapp.com/icons/{}/{}.png", guild.id, icon))
            .unwrap_or_else(|| "https://cdn.discordapp.com/embed/avatars/0.png".to_string());

        guilds_html.push_str(&format!(
            r#"<div class="server-card" onclick="location.href='/dashboard/{}'">
                <img src="{}" alt="{}" class="server-icon">
                <div class="server-info">
                    <h3>{}</h3>
                    <p>{} permission</p>
                </div>
            </div>"#,
            guild.id, icon_url, guild.name, guild.name,
            if guild.owner { "Owner" } else { "Manage Roles" }
        ));
    }

    if guilds_html.is_empty() {
        guilds_html = if !user.guilds.is_empty() {
            r#"<div class="no-servers">
                <h3>No manageable servers found</h3>
                <p>You need "Manage Roles" permission in a server to configure self-roles.</p>
                <p><a href="https://discord.com/developers/applications" target="_blank">Invite the bot to your server</a></p>
            </div>"#.to_string()
        } else {
            r#"<div class="no-servers">
                <h3>Guilds could not be loaded</h3>
                <p>There was an error loading your Discord servers. This might be a temporary issue.</p>
                <p>You are successfully logged in as a user, but guild data couldn't be retrieved.</p>
                <p><a href="/auth/logout">Logout and try again</a></p>
            </div>"#.to_string()
        };
    }

    let user_avatar = user.user.avatar.as_ref()
        .map(|avatar| format!("https://cdn.discordapp.com/avatars/{}/{}.png", user.user.id, avatar))
        .unwrap_or_else(|| "https://cdn.discordapp.com/embed/avatars/0.png".to_string());

    let template = include_str!("templates/server_list.html")
        .replace("{{COMMON_CSS}}", include_str!("static/css/common.css"))
        .replace("{{DASHBOARD_CSS}}", include_str!("static/css/dashboard.css"))
        .replace("{{USER_AVATAR}}", &user_avatar)
        .replace("{{USER_NAME}}", &user.user.username)
        .replace("{{GUILDS_HTML}}", &guilds_html);

    Ok(Html(template))
}

pub async fn guild_dashboard(
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    State(_state): State<AppState>,
) -> Result<Html<String>, Redirect> {
    let session = extract_session_data(&headers).await
        .map_err(|_| Redirect::temporary("/auth/login"))?;

    let user = session.1.ok_or_else(|| Redirect::temporary("/auth/login"))?;

    if !user.has_manage_roles_in_guild(&guild_id) {
        return Err(Redirect::temporary("/"));
    }

    let guild = user.guilds.iter().find(|g| g.id == guild_id).unwrap();

    let guild_icon = guild.icon.as_ref()
        .map(|icon| format!("https://cdn.discordapp.com/icons/{}/{}.png", guild.id, icon))
        .unwrap_or_else(|| "https://cdn.discordapp.com/embed/avatars/0.png".to_string());

    let template = include_str!("templates/guild_dashboard.html")
        .replace("{{COMMON_CSS}}", include_str!("static/css/common.css"))
        .replace("{{DASHBOARD_CSS}}", include_str!("static/css/dashboard.css"))
        .replace("{{GUILD_NAME}}", &guild.name)
        .replace("{{GUILD_ICON}}", &guild_icon)
        .replace("{{GUILD_ID}}", &guild_id);

    Ok(Html(template))
}

pub async fn selfroles_list(
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    State(_state): State<AppState>,
) -> Result<Html<String>, Redirect> {
    let session = extract_session_data(&headers).await
        .map_err(|_| Redirect::temporary("/auth/login"))?;

    let user = session.1.ok_or_else(|| Redirect::temporary("/auth/login"))?;

    if !user.has_manage_roles_in_guild(&guild_id) {
        return Err(Redirect::temporary("/"));
    }

    let guild = user.guilds.iter().find(|g| g.id == guild_id).unwrap();

    let template = include_str!("templates/selfroles_list.html")
        .replace("{{COMMON_CSS}}", include_str!("static/css/common.css"))
        .replace("{{DASHBOARD_CSS}}", include_str!("static/css/dashboard.css"))
        .replace("{{COMMON_JS}}", include_str!("static/js/common.js"))
        .replace("{{SELFROLES_JS}}", include_str!("static/js/selfroles.js"))
        .replace("{{GUILD_NAME}}", &guild.name)
        .replace("{{GUILD_ID}}", &guild_id);

    Ok(Html(template))
}

pub async fn selfroles_create(
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    State(_state): State<AppState>,
) -> Result<Html<String>, Redirect> {
    let session = extract_session_data(&headers).await
        .map_err(|_| Redirect::temporary("/auth/login"))?;

    let user = session.1.ok_or_else(|| Redirect::temporary("/auth/login"))?;

    if !user.has_manage_roles_in_guild(&guild_id) {
        return Err(Redirect::temporary("/"));
    }

    let guild = user.guilds.iter().find(|g| g.id == guild_id).unwrap();
    render_selfroles_form(&guild_id, &guild.name, None)
}

pub async fn selfroles_edit(
    Path((guild_id, config_id)): Path<(String, String)>,
    headers: HeaderMap,
    State(_state): State<AppState>,
) -> Result<Html<String>, Redirect> {
    let session = extract_session_data(&headers).await
        .map_err(|_| Redirect::temporary("/auth/login"))?;

    let user = session.1.ok_or_else(|| Redirect::temporary("/auth/login"))?;

    if !user.has_manage_roles_in_guild(&guild_id) {
        return Err(Redirect::temporary("/"));
    }

    let guild = user.guilds.iter().find(|g| g.id == guild_id).unwrap();
    render_selfroles_form(&guild_id, &guild.name, Some(&config_id))
}

fn render_selfroles_form(
    guild_id: &str,
    guild_name: &str,
    config_id: Option<&str>,
) -> Result<Html<String>, Redirect> {
    let (page_title, header_title, header_description, breadcrumb_current, button_text, config_id_script, config_id_param) = 
        if let Some(config_id) = config_id {
            (
                "Edit Self-Role Message",
                "Edit Self-Role Message", 
                "Edit interactive role assignment message for",
                "Edit",
                "Update Self-Role Message",
                format!("const configId = '{}';", config_id),
                ", configId"
            )
        } else {
            (
                "Create Self-Role Message",
                "Create Self-Role Message",
                "Create a new interactive role assignment message for", 
                "Create",
                "Deploy Self-Role Message",
                "".to_string(),
                ""
            )
        };
    
    let template = include_str!("templates/selfroles_form.html")
        .replace("{{COMMON_CSS}}", include_str!("static/css/common.css"))
        .replace("{{DASHBOARD_CSS}}", include_str!("static/css/dashboard.css"))
        .replace("{{COMMON_JS}}", include_str!("static/js/common.js"))
        .replace("{{SELFROLES_JS}}", include_str!("static/js/selfroles.js"))
        .replace("{{GUILD_NAME}}", guild_name)
        .replace("{{GUILD_ID}}", guild_id)
        .replace("{{PAGE_TITLE}}", page_title)
        .replace("{{HEADER_TITLE}}", header_title)
        .replace("{{HEADER_DESCRIPTION}}", header_description)
        .replace("{{BREADCRUMB_CURRENT}}", breadcrumb_current)
        .replace("{{BUTTON_TEXT}}", button_text)
        .replace("{{CONFIG_ID_SCRIPT}}", &config_id_script)
        .replace("{{CONFIG_ID_PARAM}}", config_id_param);

    Ok(Html(template))
}
