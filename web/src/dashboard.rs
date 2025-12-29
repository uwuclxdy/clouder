use crate::{config::AppState, middleware::extract_session_data};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{Html, Redirect},
    Json,
};
use serde_json::{json, Value};

pub async fn guild_dashboard(
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Html<String>, Redirect> {
    let (_, user) = extract_session_data(&headers)
        .await
        .map_err(|_| Redirect::temporary("/auth/login"))?;
    let user = user.ok_or_else(|| Redirect::temporary("/auth/login"))?;

    let guild_info = user
        .guilds
        .iter()
        .find(|g| g.id == guild_id)
        .ok_or_else(|| Redirect::temporary("/"))?;

    if !user.has_manage_roles_in_guild(&guild_id) && !user.has_administrator_in_guild(&guild_id) {
        return Err(Redirect::temporary("/"));
    }

    let guild_icon = guild_info
        .icon
        .as_ref()
        .map(|hash| format!("https://cdn.discordapp.com/icons/{}/{}.png", guild_id, hash))
        .unwrap_or_else(|| "https://cdn.discordapp.com/embed/avatars/0.png".to_string());

    let template = include_str!("../templates/guild_dashboard.html")
        .replace("{{COMMON_CSS}}", include_str!("../static/css/common.css"))
        .replace(
            "{{DASHBOARD_CSS}}",
            include_str!("../static/css/dashboard.css"),
        )
        .replace("{{GUILD_NAME}}", &guild_info.name)
        .replace("{{GUILD_ICON}}", &guild_icon)
        .replace("{{GUILD_ID}}", &guild_id)
        .replace("{{BASE_URL}}", &state.config.web.base_url)
        .replace(
            "{{INVITE_URL}}",
            &format!(
                "https://discord.com/api/oauth2/authorize?client_id={}&permissions=8&scope=bot",
                state.config.web.oauth.client_id
            ),
        );

    Ok(Html(template))
}

pub async fn feature_request(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Html<String>, Redirect> {
    let (_, user) = extract_session_data(&headers)
        .await
        .map_err(|_| Redirect::temporary("/auth/login"))?;
    let user = user.ok_or_else(|| Redirect::temporary("/auth/login"))?;

    let user_avatar = user
        .user
        .avatar
        .as_ref()
        .map(|hash| {
            format!(
                "https://cdn.discordapp.com/avatars/{}/{}.png",
                user.user.id, hash
            )
        })
        .unwrap_or_else(|| {
            format!(
                "https://cdn.discordapp.com/embed/avatars/{}.png",
                (user.user.id.parse::<u64>().unwrap_or(0) % 5) as u8
            )
        });

    let template = include_str!("../templates/feature_request.html")
        .replace("{{COMMON_CSS}}", include_str!("../static/css/common.css"))
        .replace(
            "{{DASHBOARD_CSS}}",
            include_str!("../static/css/dashboard.css"),
        )
        .replace("{{COMMON_JS}}", include_str!("../static/js/common.js"))
        .replace("{{USERNAME}}", &user.user.username)
        .replace("{{USER_NAME}}", &user.user.username)
        .replace("{{USER_AVATAR}}", &user_avatar)
        .replace("{{BASE_URL}}", &state.config.web.base_url)
        .replace(
            "{{INVITE_URL}}",
            &format!(
                "https://discord.com/api/oauth2/authorize?client_id={}&permissions=8&scope=bot",
                state.config.web.oauth.client_id
            ),
        );

    Ok(Html(template))
}

pub async fn user_settings(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Html<String>, Redirect> {
    let (_, user) = extract_session_data(&headers)
        .await
        .map_err(|_| Redirect::temporary("/auth/login"))?;
    let user = user.ok_or_else(|| Redirect::temporary("/auth/login"))?;

    let template = include_str!("../templates/server_list.html")
        .replace("{{COMMON_CSS}}", include_str!("../static/css/common.css"))
        .replace(
            "{{DASHBOARD_CSS}}",
            include_str!("../static/css/dashboard.css"),
        )
        .replace("{{USERNAME}}", &user.user.username)
        .replace("{{USER_NAME}}", &user.user.username)
        .replace(
            "{{USER_AVATAR}}",
            &format!(
                "https://cdn.discordapp.com/embed/avatars/{}.png",
                (user.user.id.parse::<u64>().unwrap_or(0) % 5) as u8
            ),
        )
        .replace("{{GUILDS_HTML}}", "")
        .replace("{{BASE_URL}}", &state.config.web.base_url)
        .replace("{{INVITE_URL}}", "");

    Ok(Html(template))
}

pub async fn api_get_channels(
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let (_, user) = extract_session_data(&headers)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let user = user.ok_or(StatusCode::UNAUTHORIZED)?;

    if !user.has_manage_roles_in_guild(&guild_id) && !user.has_administrator_in_guild(&guild_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    let _guild_id_u64: u64 = guild_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    // For now, return placeholder data since we don't have clouder integration
    Ok(Json(json!({
        "message": "API functionality not yet implemented - clouder integration needed",
        "guild_id": guild_id
    })))
}

pub async fn api_get_roles(
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let (_, user) = extract_session_data(&headers)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let user = user.ok_or(StatusCode::UNAUTHORIZED)?;

    if !user.has_manage_roles_in_guild(&guild_id) && !user.has_administrator_in_guild(&guild_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    let _guild_id_u64: u64 = guild_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    // For now, return placeholder data since we don't have clouder integration
    Ok(Json(json!({
        "message": "API functionality not yet implemented - clouder integration needed",
        "guild_id": guild_id
    })))
}

pub async fn api_selfroles_list(
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let (_, user) = extract_session_data(&headers)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let user = user.ok_or(StatusCode::UNAUTHORIZED)?;

    if !user.has_manage_roles_in_guild(&guild_id) && !user.has_administrator_in_guild(&guild_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    let _guild_id_u64: u64 = guild_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    // For now, return placeholder data since we don't have clouder integration
    Ok(Json(json!({
        "message": "API functionality not yet implemented - clouder integration needed",
        "guild_id": guild_id
    })))
}

pub async fn api_selfroles_create(
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    State(_state): State<AppState>,
    Json(mut payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let (_, user) = extract_session_data(&headers)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let user = user.ok_or(StatusCode::UNAUTHORIZED)?;

    if !user.has_manage_roles_in_guild(&guild_id) && !user.has_administrator_in_guild(&guild_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    let user_id: u64 = user.user.id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    payload["user_id"] = json!(user_id);

    let _guild_id_u64: u64 = guild_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    // For now, return placeholder data since we don't have clouder integration
    Ok(Json(json!({
        "message": "API functionality not yet implemented - clouder integration needed",
        "guild_id": guild_id,
        "payload": payload
    })))
}

pub async fn api_selfroles_delete(
    Path((guild_id, config_id)): Path<(String, String)>,
    headers: HeaderMap,
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let (_, user) = extract_session_data(&headers)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let user = user.ok_or(StatusCode::UNAUTHORIZED)?;

    if !user.has_manage_roles_in_guild(&guild_id) && !user.has_administrator_in_guild(&guild_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    let _guild_id_u64: u64 = guild_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    let _config_id_i64: i64 = config_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    // For now, return placeholder data since we don't have clouder integration
    Ok(Json(json!({
        "message": "API functionality not yet implemented - clouder integration needed",
        "guild_id": guild_id,
        "config_id": config_id
    })))
}

pub async fn selfroles_list(
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Html<String>, Redirect> {
    let (_, user) = extract_session_data(&headers)
        .await
        .map_err(|_| Redirect::temporary("/auth/login"))?;
    let user = user.ok_or_else(|| Redirect::temporary("/auth/login"))?;

    if !user.has_manage_roles_in_guild(&guild_id) && !user.has_administrator_in_guild(&guild_id) {
        return Err(Redirect::temporary("/"));
    }

    let guild_info = user
        .guilds
        .iter()
        .find(|g| g.id == guild_id)
        .ok_or_else(|| Redirect::temporary("/"))?;

    let guild_icon = guild_info
        .icon
        .as_ref()
        .map(|hash| format!("https://cdn.discordapp.com/icons/{}/{}.png", guild_id, hash))
        .unwrap_or_else(|| "https://cdn.discordapp.com/embed/avatars/0.png".to_string());

    let template = include_str!("../templates/selfroles_list.html")
        .replace("{{COMMON_CSS}}", include_str!("../static/css/common.css"))
        .replace(
            "{{DASHBOARD_CSS}}",
            include_str!("../static/css/dashboard.css"),
        )
        .replace("{{COMMON_JS}}", include_str!("../static/js/common.js"))
        .replace(
            "{{SELFROLES_JS}}",
            include_str!("../static/js/selfroles.js"),
        )
        .replace("{{GUILD_NAME}}", &guild_info.name)
        .replace("{{GUILD_ICON}}", &guild_icon)
        .replace("{{GUILD_ID}}", &guild_id)
        .replace(
            "{{INVITE_URL}}",
            &format!(
                "https://discord.com/api/oauth2/authorize?client_id={}&permissions=8&scope=bot",
                state.config.web.oauth.client_id
            ),
        )
        .replace("{{BASE_URL}}", &state.config.web.base_url);

    Ok(Html(template))
}

pub async fn selfroles_new(
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Html<String>, Redirect> {
    let (_, user) = extract_session_data(&headers)
        .await
        .map_err(|_| Redirect::temporary("/auth/login"))?;
    let user = user.ok_or_else(|| Redirect::temporary("/auth/login"))?;

    if !user.has_manage_roles_in_guild(&guild_id) && !user.has_administrator_in_guild(&guild_id) {
        return Err(Redirect::temporary("/"));
    }

    let guild_info = user
        .guilds
        .iter()
        .find(|g| g.id == guild_id)
        .ok_or_else(|| Redirect::temporary("/"))?;

    let guild_icon = guild_info
        .icon
        .as_ref()
        .map(|hash| format!("https://cdn.discordapp.com/icons/{}/{}.png", guild_id, hash))
        .unwrap_or_else(|| "https://cdn.discordapp.com/embed/avatars/0.png".to_string());

    let template = include_str!("../templates/selfroles_form.html")
        .replace("{{COMMON_CSS}}", include_str!("../static/css/common.css"))
        .replace(
            "{{DASHBOARD_CSS}}",
            include_str!("../static/css/dashboard.css"),
        )
        .replace("{{COMMON_JS}}", include_str!("../static/js/common.js"))
        .replace(
            "{{SELFROLES_CONFIG_JS}}",
            include_str!("../static/js/selfroles_config.js"),
        )
        .replace("{{GUILD_NAME}}", &guild_info.name)
        .replace("{{GUILD_ICON}}", &guild_icon)
        .replace("{{GUILD_ID}}", &guild_id)
        .replace(
            "{{INVITE_URL}}",
            &format!(
                "https://discord.com/api/oauth2/authorize?client_id={}&permissions=8&scope=bot",
                state.config.web.oauth.client_id
            ),
        )
        .replace("{{BASE_URL}}", &state.config.web.base_url)
        .replace("{{CONFIG_ID}}", "new");

    Ok(Html(template))
}

pub async fn selfroles_edit(
    Path((guild_id, config_id)): Path<(String, String)>,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Html<String>, Redirect> {
    let (_, user) = extract_session_data(&headers)
        .await
        .map_err(|_| Redirect::temporary("/auth/login"))?;
    let user = user.ok_or_else(|| Redirect::temporary("/auth/login"))?;

    if !user.has_manage_roles_in_guild(&guild_id) && !user.has_administrator_in_guild(&guild_id) {
        return Err(Redirect::temporary("/"));
    }

    let guild_info = user
        .guilds
        .iter()
        .find(|g| g.id == guild_id)
        .ok_or_else(|| Redirect::temporary("/"))?;

    let guild_icon = guild_info
        .icon
        .as_ref()
        .map(|hash| format!("https://cdn.discordapp.com/icons/{}/{}.png", guild_id, hash))
        .unwrap_or_else(|| "https://cdn.discordapp.com/embed/avatars/0.png".to_string());

    let template = include_str!("../templates/selfroles_form.html")
        .replace("{{COMMON_CSS}}", include_str!("../static/css/common.css"))
        .replace(
            "{{DASHBOARD_CSS}}",
            include_str!("../static/css/dashboard.css"),
        )
        .replace("{{COMMON_JS}}", include_str!("../static/js/common.js"))
        .replace(
            "{{SELFROLES_CONFIG_JS}}",
            include_str!("../static/js/selfroles_config.js"),
        )
        .replace("{{GUILD_NAME}}", &guild_info.name)
        .replace("{{GUILD_ICON}}", &guild_icon)
        .replace("{{GUILD_ID}}", &guild_id)
        .replace(
            "{{INVITE_URL}}",
            &format!(
                "https://discord.com/api/oauth2/authorize?client_id={}&permissions=8&scope=bot",
                state.config.web.oauth.client_id
            ),
        )
        .replace("{{BASE_URL}}", &state.config.web.base_url)
        .replace("{{CONFIG_ID}}", &config_id);

    Ok(Html(template))
}

pub async fn welcome_goodbye_config(
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Html<String>, Redirect> {
    let (_, user) = extract_session_data(&headers)
        .await
        .map_err(|_| Redirect::temporary("/auth/login"))?;
    let user = user.ok_or_else(|| Redirect::temporary("/auth/login"))?;

    if !user.has_manage_roles_in_guild(&guild_id) && !user.has_administrator_in_guild(&guild_id) {
        return Err(Redirect::temporary("/"));
    }

    let guild_info = user
        .guilds
        .iter()
        .find(|g| g.id == guild_id)
        .ok_or_else(|| Redirect::temporary("/"))?;

    let guild_icon = guild_info
        .icon
        .as_ref()
        .map(|hash| format!("https://cdn.discordapp.com/icons/{}/{}.png", guild_id, hash))
        .unwrap_or_else(|| "https://cdn.discordapp.com/embed/avatars/0.png".to_string());

    let template = include_str!("../templates/welcome_goodbye_config.html")
        .replace("{{COMMON_CSS}}", include_str!("../static/css/common.css"))
        .replace(
            "{{DASHBOARD_CSS}}",
            include_str!("../static/css/dashboard.css"),
        )
        .replace("{{COMMON_JS}}", include_str!("../static/js/common.js"))
        .replace("{{GUILD_NAME}}", &guild_info.name)
        .replace("{{GUILD_ICON}}", &guild_icon)
        .replace("{{GUILD_ID}}", &guild_id)
        .replace(
            "{{INVITE_URL}}",
            &format!(
                "https://discord.com/api/oauth2/authorize?client_id={}&permissions=8&scope=bot",
                state.config.web.oauth.client_id
            ),
        )
        .replace("{{BASE_URL}}", &state.config.web.base_url)
        .replace("{{CHANNELS_HTML}}", "")
        .replace("{{WELCOME_ENABLED}}", "")
        .replace("{{GOODBYE_ENABLED}}", "")
        .replace("{{WELCOME_CONFIG_DISPLAY}}", "none")
        .replace("{{GOODBYE_CONFIG_DISPLAY}}", "none")
        .replace("{{WELCOME_MESSAGE_TYPE_EMBED_CLASS}}", "")
        .replace("{{WELCOME_MESSAGE_TYPE_TEXT_CLASS}}", "")
        .replace("{{WELCOME_MESSAGE_TYPE_EMBED_CHECKED}}", "")
        .replace("{{WELCOME_MESSAGE_TYPE_TEXT_CHECKED}}", "")
        .replace("{{WELCOME_TEXT_HIDE}}", "")
        .replace("{{WELCOME_EMBED_SHOW}}", "")
        .replace("{{WELCOME_MESSAGE_CONTENT}}", "")
        .replace("{{WELCOME_EMBED_TITLE}}", "#5865F2")
        .replace("{{WELCOME_EMBED_DESCRIPTION}}", "")
        .replace("{{WELCOME_EMBED_COLOR}}", "#5865F2")
        .replace("{{WELCOME_EMBED_FOOTER}}", "")
        .replace("{{WELCOME_EMBED_THUMBNAIL}}", "")
        .replace("{{WELCOME_EMBED_IMAGE}}", "")
        .replace("{{WELCOME_EMBED_TIMESTAMP}}", "")
        .replace("{{GOODBYE_MESSAGE_TYPE_EMBED_CLASS}}", "")
        .replace("{{GOODBYE_MESSAGE_TYPE_TEXT_CLASS}}", "")
        .replace("{{GOODBYE_MESSAGE_TYPE_EMBED_CHECKED}}", "")
        .replace("{{GOODBYE_MESSAGE_TYPE_TEXT_CHECKED}}", "")
        .replace("{{GOODBYE_TEXT_HIDE}}", "")
        .replace("{{GOODBYE_EMBED_SHOW}}", "")
        .replace("{{GOODBYE_MESSAGE_CONTENT}}", "")
        .replace("{{GOODBYE_EMBED_TITLE}}", "#5865F2")
        .replace("{{GOODBYE_EMBED_DESCRIPTION}}", "")
        .replace("{{GOODBYE_EMBED_COLOR}}", "#5865F2")
        .replace("{{GOODBYE_EMBED_FOOTER}}", "")
        .replace("{{GOODBYE_EMBED_THUMBNAIL}}", "")
        .replace("{{GOODBYE_EMBED_IMAGE}}", "")
        .replace("{{GOODBYE_EMBED_TIMESTAMP}}", "");

    Ok(Html(template))
}

pub async fn mediaonly_config(
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Html<String>, Redirect> {
    let (_, user) = extract_session_data(&headers)
        .await
        .map_err(|_| Redirect::temporary("/auth/login"))?;
    let user = user.ok_or_else(|| Redirect::temporary("/auth/login"))?;

    if !user.has_manage_roles_in_guild(&guild_id) && !user.has_administrator_in_guild(&guild_id) {
        return Err(Redirect::temporary("/"));
    }

    let guild_info = user
        .guilds
        .iter()
        .find(|g| g.id == guild_id)
        .ok_or_else(|| Redirect::temporary("/"))?;

    let guild_icon = guild_info
        .icon
        .as_ref()
        .map(|hash| format!("https://cdn.discordapp.com/icons/{}/{}.png", guild_id, hash))
        .unwrap_or_else(|| "https://cdn.discordapp.com/embed/avatars/0.png".to_string());

    let template = include_str!("../templates/mediaonly_config.html")
        .replace("{{COMMON_CSS}}", include_str!("../static/css/common.css"))
        .replace(
            "{{DASHBOARD_CSS}}",
            include_str!("../static/css/dashboard.css"),
        )
        .replace("{{COMMON_JS}}", include_str!("../static/js/common.js"))
        .replace("{{GUILD_NAME}}", &guild_info.name)
        .replace("{{GUILD_ICON}}", &guild_icon)
        .replace("{{GUILD_ID}}", &guild_id)
        .replace(
            "{{INVITE_URL}}",
            &format!(
                "https://discord.com/api/oauth2/authorize?client_id={}&permissions=8&scope=bot",
                state.config.web.oauth.client_id
            ),
        )
        .replace("{{BASE_URL}}", &state.config.web.base_url)
        .replace("{{CHANNELS_HTML}}", "")
        .replace("{{CONFIGS_JSON}}", "[]");

    Ok(Html(template))
}

// API handlers
pub async fn api_welcome_goodbye_get(
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    State(_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let (_, user) = extract_session_data(&headers)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let user = user.ok_or(StatusCode::UNAUTHORIZED)?;

    if !user.has_manage_roles_in_guild(&guild_id) && !user.has_administrator_in_guild(&guild_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(Json(json!({
        "success": true,
        "message": "Welcome/Goodbye API not yet implemented"
    })))
}

pub async fn api_welcome_goodbye_post(
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    State(_state): State<AppState>,
    Json(_payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let (_, user) = extract_session_data(&headers)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let user = user.ok_or(StatusCode::UNAUTHORIZED)?;

    if !user.has_manage_roles_in_guild(&guild_id) && !user.has_administrator_in_guild(&guild_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(Json(json!({
        "success": true,
        "message": "Welcome/Goodbye configuration saved (placeholder)"
    })))
}

pub async fn api_welcome_goodbye_test(
    Path((guild_id, message_type)): Path<(String, String)>,
    headers: HeaderMap,
    State(_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let (_, user) = extract_session_data(&headers)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let user = user.ok_or(StatusCode::UNAUTHORIZED)?;

    if !user.has_manage_roles_in_guild(&guild_id) && !user.has_administrator_in_guild(&guild_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(Json(json!({
        "success": true,
        "message": format!("Test {} message sent (placeholder)", message_type)
    })))
}

pub async fn api_mediaonly_get(
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    State(_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let (_, user) = extract_session_data(&headers)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let user = user.ok_or(StatusCode::UNAUTHORIZED)?;

    if !user.has_manage_roles_in_guild(&guild_id) && !user.has_administrator_in_guild(&guild_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(Json(json!({
        "success": true,
        "configs": []
    })))
}

pub async fn api_mediaonly_post(
    Path(guild_id): Path<String>,
    headers: HeaderMap,
    State(_state): State<AppState>,
    Json(_payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let (_, user) = extract_session_data(&headers)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let user = user.ok_or(StatusCode::UNAUTHORIZED)?;

    if !user.has_manage_roles_in_guild(&guild_id) && !user.has_administrator_in_guild(&guild_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(Json(json!({
        "success": true,
        "message": "Media-only channel added (placeholder)"
    })))
}

pub async fn api_mediaonly_delete(
    Path((guild_id, _channel_id)): Path<(String, String)>,
    headers: HeaderMap,
    State(_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let (_, user) = extract_session_data(&headers)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let user = user.ok_or(StatusCode::UNAUTHORIZED)?;

    if !user.has_manage_roles_in_guild(&guild_id) && !user.has_administrator_in_guild(&guild_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(Json(json!({
        "success": true,
        "message": "Media-only channel removed (placeholder)"
    })))
}

pub async fn api_mediaonly_put(
    Path((guild_id, _channel_id)): Path<(String, String)>,
    headers: HeaderMap,
    State(_state): State<AppState>,
    Json(_payload): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let (_, user) = extract_session_data(&headers)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let user = user.ok_or(StatusCode::UNAUTHORIZED)?;

    if !user.has_manage_roles_in_guild(&guild_id) && !user.has_administrator_in_guild(&guild_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(Json(json!({
        "success": true,
        "message": "Media-only channel updated (placeholder)"
    })))
}
