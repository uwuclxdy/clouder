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
    let session = match extract_session_data(&headers).await {
        Ok(session) => session,
        Err(_) => return Err(Redirect::temporary("/auth/login")),
    };

    let user = match session.1 {
        Some(user) => user,
        None => return Err(Redirect::temporary("/auth/login")),
    };

    let manageable_guilds = user.get_manageable_guilds();

    let mut guilds_html = String::new();
    for guild in manageable_guilds {
        let icon_url = if let Some(icon) = &guild.icon {
            format!("https://cdn.discordapp.com/icons/{}/{}.png", guild.id, icon)
        } else {
            "https://cdn.discordapp.com/embed/avatars/0.png".to_string()
        };

        guilds_html.push_str(&format!(
            r#"
            <div class="server-card" onclick="location.href='/dashboard/{}'">
                <img src="{}" alt="{}" class="server-icon">
                <div class="server-info">
                    <h3>{}</h3>
                    <p>{} permission</p>
                </div>
            </div>
            "#,
            guild.id,
            icon_url,
            guild.name,
            guild.name,
            if guild.owner { "Owner" } else { "Manage Roles" }
        ));
    }

    if guilds_html.is_empty() {
        let has_guilds = !user.guilds.is_empty();
        guilds_html = if has_guilds {
            r#"
            <div class="no-servers">
                <h3>No manageable servers found</h3>
                <p>You need "Manage Roles" permission in a server to configure self-roles.</p>
                <p><a href="https://discord.com/developers/applications" target="_blank">Invite the bot to your server</a></p>
            </div>
            "#.to_string()
        } else {
            r#"
            <div class="no-servers">
                <h3>Guilds could not be loaded</h3>
                <p>There was an error loading your Discord servers. This might be a temporary issue.</p>
                <p>You are successfully logged in as a user, but guild data couldn't be retrieved.</p>
                <p><a href="/auth/logout">Logout and try again</a></p>
            </div>
            "#.to_string()
        };
    }

    let user_avatar = if let Some(avatar) = &user.user.avatar {
        format!("https://cdn.discordapp.com/avatars/{}/{}.png", user.user.id, avatar)
    } else {
        "https://cdn.discordapp.com/embed/avatars/0.png".to_string()
    };

    let common_css = include_str!("static/css/common.css");
    let dashboard_css = include_str!("static/css/dashboard.css");
    let template = include_str!("templates/server_list.html");
    
    let html = template
        .replace("{{COMMON_CSS}}", common_css)
        .replace("{{DASHBOARD_CSS}}", dashboard_css)
        .replace("{{USER_AVATAR}}", &user_avatar)
        .replace("{{USER_NAME}}", &user.user.username)
        .replace("{{GUILDS_HTML}}", &guilds_html);

    Ok(Html(html))
}

pub async fn guild_dashboard(
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    State(_state): State<AppState>,
) -> Result<Html<String>, Redirect> {
    let session = match extract_session_data(&headers).await {
        Ok(session) => session,
        Err(_) => return Err(Redirect::temporary("/auth/login")),
    };

    let user = match session.1 {
        Some(user) => user,
        None => return Err(Redirect::temporary("/auth/login")),
    };

    if !user.has_manage_roles_in_guild(&guild_id) {
        return Err(Redirect::temporary("/"));
    }

    let guild = user.guilds.iter().find(|g| g.id == guild_id).unwrap();

    let guild_icon = if let Some(icon) = &guild.icon {
        format!("https://cdn.discordapp.com/icons/{}/{}.png", guild.id, icon)
    } else {
        "https://cdn.discordapp.com/embed/avatars/0.png".to_string()
    };

    let common_css = include_str!("static/css/common.css");
    let dashboard_css = include_str!("static/css/dashboard.css");
    let template = include_str!("templates/guild_dashboard.html");
    
    let html = template
        .replace("{{COMMON_CSS}}", common_css)
        .replace("{{DASHBOARD_CSS}}", dashboard_css)
        .replace("{{GUILD_NAME}}", &guild.name)
        .replace("{{GUILD_ICON}}", &guild_icon)
        .replace("{{GUILD_ID}}", &guild_id);

    Ok(Html(html))
}

pub async fn selfroles_list(
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    State(_state): State<AppState>,
) -> Result<Html<String>, Redirect> {
    let session = match extract_session_data(&headers).await {
        Ok(session) => session,
        Err(_) => return Err(Redirect::temporary("/auth/login")),
    };

    let user = match session.1 {
        Some(user) => user,
        None => return Err(Redirect::temporary("/auth/login")),
    };

    if !user.has_manage_roles_in_guild(&guild_id) {
        return Err(Redirect::temporary("/"));
    }

    let guild = user.guilds.iter().find(|g| g.id == guild_id).unwrap();

    let common_css = include_str!("static/css/common.css");
    let dashboard_css = include_str!("static/css/dashboard.css");
    let common_js = include_str!("static/js/common.js");
    let selfroles_js = include_str!("static/js/selfroles.js");
    let template = include_str!("templates/selfroles_list.html");
    
    let html = template
        .replace("{{COMMON_CSS}}", common_css)
        .replace("{{DASHBOARD_CSS}}", dashboard_css)
        .replace("{{COMMON_JS}}", common_js)
        .replace("{{SELFROLES_JS}}", selfroles_js)
        .replace("{{GUILD_NAME}}", &guild.name)
        .replace("{{GUILD_ID}}", &guild_id);

    Ok(Html(html))
}

pub async fn selfroles_create(
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    State(_state): State<AppState>,
) -> Result<Html<String>, Redirect> {
    let session = match extract_session_data(&headers).await {
        Ok(session) => session,
        Err(_) => return Err(Redirect::temporary("/auth/login")),
    };

    let user = match session.1 {
        Some(user) => user,
        None => return Err(Redirect::temporary("/auth/login")),
    };

    if !user.has_manage_roles_in_guild(&guild_id) {
        return Err(Redirect::temporary("/"));
    }

    let guild = user.guilds.iter().find(|g| g.id == guild_id).unwrap();

    let common_css = include_str!("static/css/common.css");
    let dashboard_css = include_str!("static/css/dashboard.css");
    let common_js = include_str!("static/js/common.js");
    let selfroles_js = include_str!("static/js/selfroles.js");
    let template = include_str!("templates/selfroles_create.html");
    
    let html = template
        .replace("{{COMMON_CSS}}", common_css)
        .replace("{{DASHBOARD_CSS}}", dashboard_css)
        .replace("{{COMMON_JS}}", common_js)
        .replace("{{SELFROLES_JS}}", selfroles_js)
        .replace("{{GUILD_NAME}}", &guild.name)
        .replace("{{GUILD_ID}}", &guild_id);

    Ok(Html(html))
}

pub async fn selfroles_edit(
    Path((guild_id, config_id)): Path<(String, String)>,
    headers: HeaderMap,
    State(_state): State<AppState>,
) -> Result<Html<String>, Redirect> {
    let session = match extract_session_data(&headers).await {
        Ok(session) => session,
        Err(_) => return Err(Redirect::temporary("/auth/login")),
    };

    let user = match session.1 {
        Some(user) => user,
        None => return Err(Redirect::temporary("/auth/login")),
    };

    if !user.has_manage_roles_in_guild(&guild_id) {
        return Err(Redirect::temporary("/"));
    }

    let guild = user.guilds.iter().find(|g| g.id == guild_id).unwrap();

    let common_css = include_str!("static/css/common.css");
    let dashboard_css = include_str!("static/css/dashboard.css");
    let common_js = include_str!("static/js/common.js");
    let selfroles_js = include_str!("static/js/selfroles.js");
    let template = include_str!("templates/selfroles_edit.html");
    
    let html = template
        .replace("{{COMMON_CSS}}", common_css)
        .replace("{{DASHBOARD_CSS}}", dashboard_css)
        .replace("{{COMMON_JS}}", common_js)
        .replace("{{SELFROLES_JS}}", selfroles_js)
        .replace("{{GUILD_NAME}}", &guild.name)
        .replace("{{GUILD_ID}}", &guild_id)
        .replace("{{CONFIG_ID}}", &config_id);

    Ok(Html(html))
}
