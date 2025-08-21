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

        let guild_card = include_str!("templates/partials/guild_card.html")
            .replace("{{GUILD_ID}}", &guild.id)
            .replace("{{ICON_URL}}", &icon_url)
            .replace("{{GUILD_NAME}}", &guild.name)
            .replace("{{PERMISSION_TEXT}}", if guild.owner { "Owner" } else { "Manage Roles" });

        guilds_html.push_str(&guild_card);
    }

    if guilds_html.is_empty() {
        guilds_html = if !user.guilds.is_empty() {
            include_str!("templates/partials/no_manageable_servers.html").to_string()
        } else {
            include_str!("templates/partials/guild_load_error.html").to_string()
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
    let (page_title, header_title, header_description, breadcrumb_current, button_text) =
        if config_id.is_some() {
            (
                "Edit Self-Role Message",
                "Edit Self-Role Message",
                "Edit interactive role assignment message for",
                "Edit",
                "Update Self-Role Message"
            )
        } else {
            (
                "Create Self-Role Message",
                "Create Self-Role Message",
                "Create a new interactive role assignment message for",
                "Create",
                "Deploy Self-Role Message"
            )
        };

    let template = include_str!("templates/selfroles_form.html")
        .replace("{{COMMON_CSS}}", include_str!("static/css/common.css"))
        .replace("{{DASHBOARD_CSS}}", include_str!("static/css/dashboard.css"))
        .replace("{{COMMON_JS}}", include_str!("static/js/common.js"))
        .replace("{{SELFROLES_CONFIG_JS}}", include_str!("static/js/selfroles_config.js"))
        .replace("{{SELFROLES_JS}}", include_str!("static/js/selfroles.js"))
        .replace("{{GUILD_NAME}}", guild_name)
        .replace("{{GUILD_ID}}", guild_id)
        .replace("{{PAGE_TITLE}}", page_title)
        .replace("{{HEADER_TITLE}}", header_title)
        .replace("{{HEADER_DESCRIPTION}}", header_description)
        .replace("{{BREADCRUMB_CURRENT}}", breadcrumb_current)
        .replace("{{BUTTON_TEXT}}", button_text);

    Ok(Html(template))
}
