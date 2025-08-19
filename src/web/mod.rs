use crate::config::AppState;
use axum::{
    extract::{Path, State},
    http::{StatusCode, HeaderMap},
    routing::{get, post, delete, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use session_extractor::extract_session_data;

pub mod embed;
pub mod auth;
pub mod dashboard;
pub mod session;
pub mod middleware;
pub mod models;
pub mod session_extractor;

pub fn create_router(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(dashboard::server_list))
        .route("/auth/login", get(login_page))
        .route("/auth/discord", get(auth::login))
        .route("/auth/callback", get(auth::callback))
        .route("/auth/logout", get(auth::logout))
        .route("/dashboard/{guild_id}", get(dashboard::guild_dashboard))
        .route("/dashboard/{guild_id}/selfroles", get(dashboard::selfroles_list))
        .route("/dashboard/{guild_id}/selfroles/new", get(dashboard::selfroles_create))
        .route("/dashboard/{guild_id}/selfroles/edit/{config_id}", get(dashboard::selfroles_edit))
        .route("/api/selfroles/{guild_id}", get(api_get_selfroles).post(api_create_selfroles))
        .route("/api/selfroles/{guild_id}/{config_id}", get(api_get_selfrole_config).put(api_update_selfroles).delete(api_delete_selfroles))
        .route("/api/guild/{guild_id}/channels", get(api_get_channels))
        .route("/api/guild/{guild_id}/roles", get(api_get_roles))
        .route("/debug/sessions", get(debug_sessions))
        .layer(axum::middleware::from_fn(middleware::session_middleware))
        .with_state(app_state)
}

async fn login_page() -> axum::response::Html<&'static str> {
    axum::response::Html(r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Clouder Bot - Login</title>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <style>
            body {
                font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                margin: 0;
                padding: 20px;
                min-height: 100vh;
                display: flex;
                align-items: center;
                justify-content: center;
            }
            .login-container {
                background: rgba(255, 255, 255, 0.1);
                backdrop-filter: blur(10px);
                border-radius: 15px;
                padding: 40px;
                text-align: center;
                color: white;
                max-width: 400px;
                width: 100%;
            }
            h1 {
                margin: 0 0 20px 0;
                font-size: 2.5em;
                font-weight: 300;
            }
            p {
                margin-bottom: 30px;
                opacity: 0.9;
                line-height: 1.6;
            }
            .login-btn {
                background: #5865F2;
                color: white;
                padding: 15px 30px;
                border: none;
                border-radius: 8px;
                font-size: 16px;
                font-weight: 500;
                cursor: pointer;
                text-decoration: none;
                display: inline-block;
                transition: all 0.3s ease;
            }
            .login-btn:hover {
                background: #4752C4;
                transform: translateY(-2px);
                box-shadow: 0 5px 15px rgba(0, 0, 0, 0.2);
            }
        </style>
    </head>
    <body>
        <div class="login-container">
            <h1>Clouder Bot</h1>
            <p>Welcome to the Clouder Discord bot dashboard. You need to authenticate with Discord to access the dashboard and configure self-roles for your servers.</p>
            <a href="/auth/discord" class="login-btn">Login with Discord</a>
        </div>
    </body>
    </html>
    "#)
}

async fn debug_sessions(headers: HeaderMap) -> axum::response::Html<String> {
    use middleware::GLOBAL_SESSION_STORE;

    let sessions_count = GLOBAL_SESSION_STORE.session_count().await;

    let cookie_info = match headers.get(axum::http::header::COOKIE) {
        Some(cookie) => format!("Cookie header: {}", cookie.to_str().unwrap_or("invalid")),
        None => "No cookie header".to_string(),
    };

    let session_data = match extract_session_data(&headers).await {
        Ok((session, user)) => {
            format!("Session found: {}<br>User: {}",
                session.id,
                user.map(|u| u.user.username).unwrap_or_else(|| "None".to_string())
            )
        }
        Err(e) => format!("Session extraction error: {:?}", e),
    };

    let html = format!(r#"
    <!DOCTYPE html>
    <html>
    <head><title>Debug Sessions</title></head>
    <body>
        <h1>Session Debug</h1>
        <p>Total sessions in store: {}</p>
        <p>{}</p>
        <p>{}</p>
        <a href="/">Back to Home</a>
    </body>
    </html>
    "#, sessions_count, cookie_info, session_data);

    axum::response::Html(html)
}


#[derive(Serialize, Deserialize)]
struct CreateSelfRoleRequest {
    title: String,
    body: String,
    selection_type: String,
    channel_id: String,
    roles: Vec<SelfRoleData>,
}

#[derive(Serialize, Deserialize)]
struct SelfRoleData {
    role_id: String,
    emoji: String,
}


async fn api_create_selfroles(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
    Json(payload): Json<CreateSelfRoleRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let (_, user) = extract_session_data(&headers).await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let user = user.ok_or(StatusCode::UNAUTHORIZED)?;

    if !user.has_manage_roles_in_guild(&guild_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    // Validate title and body are not empty
    if payload.title.trim().is_empty() {
        return Ok(Json(serde_json::json!({
            "success": false,
            "message": "Title cannot be empty"
        })));
    }
    
    if payload.body.trim().is_empty() {
        return Ok(Json(serde_json::json!({
            "success": false,
            "message": "Body cannot be empty"
        })));
    }
    
    // Validate title and body length
    if payload.title.len() > 256 {
        return Ok(Json(serde_json::json!({
            "success": false,
            "message": "Title must be 256 characters or less"
        })));
    }
    
    if payload.body.len() > 2048 {
        return Ok(Json(serde_json::json!({
            "success": false,
            "message": "Body must be 2048 characters or less"
        })));
    }

    // Validate selection type
    if payload.selection_type != "radio" && payload.selection_type != "multiple" {
        return Ok(Json(serde_json::json!({
            "success": false,
            "message": "Invalid selection type. Must be 'radio' or 'multiple'"
        })));
    }

    // Validate roles count
    if payload.roles.is_empty() {
        return Ok(Json(serde_json::json!({
            "success": false,
            "message": "At least one role must be selected"
        })));
    }

    if payload.roles.len() > 25 {
        return Ok(Json(serde_json::json!({
            "success": false,
            "message": "Maximum 25 roles allowed per self-role message"
        })));
    }

    let guild_id_u64: u64 = guild_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    let channel_id_u64: u64 = payload.channel_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;

    // Create the database entry first
    let config = match crate::database::selfroles::SelfRoleConfig::create(
        &state.db,
        &guild_id,
        &payload.channel_id,
        &payload.title,
        &payload.body,
        &payload.selection_type,
    ).await {
        Ok(config) => config,
        Err(e) => {
            tracing::error!("Failed to create self-role config in database: {}", e);
            return Ok(Json(serde_json::json!({
                "success": false,
                "message": "Failed to save configuration"
            })));
        }
    };

    // Create role entries
    for role_data in &payload.roles {
        if let Err(e) = crate::database::selfroles::SelfRoleRole::create(
            &state.db,
            config.id,
            &role_data.role_id,
            &role_data.emoji,
        ).await {
            tracing::error!("Failed to create self-role entry: {}", e);
            // Clean up the config if role creation fails
            let _ = config.delete(&state.db).await;
            return Ok(Json(serde_json::json!({
                "success": false,
                "message": "Failed to save role configuration"
            })));
        }
    }

    // Create Discord embed and buttons
    use serenity::all::{CreateEmbed, CreateMessage, CreateActionRow, CreateButton, ButtonStyle, Colour};

    let embed = CreateEmbed::new()
        .title(&payload.title)
        .description(&payload.body)
        .colour(Colour::from_rgb(102, 126, 234)); // Match the gradient color

    // Create buttons (Discord allows max 5 buttons per row, max 5 rows)
    let mut action_rows = Vec::new();
    let mut current_row = Vec::new();

    for (index, role_data) in payload.roles.iter().enumerate() {
        let button = CreateButton::new(format!("selfrole_{}_{}", config.id, role_data.role_id))
            .label(&format!("{} {}", role_data.emoji,
                // Get role name from Discord API
                match state.http.get_guild_roles(guild_id_u64.into()).await {
                    Ok(roles) => {
                        roles.iter()
                            .find(|r| r.id.to_string() == role_data.role_id)
                            .map(|r| r.name.clone())
                            .unwrap_or_else(|| format!("Role {}", role_data.role_id))
                    }
                    Err(_) => format!("Role {}", role_data.role_id)
                }
            ))
            .style(ButtonStyle::Primary);

        current_row.push(button);

        // Discord allows 5 buttons per row
        if current_row.len() == 5 || index == payload.roles.len() - 1 {
            action_rows.push(CreateActionRow::Buttons(current_row.clone()));
            current_row.clear();

            // Discord allows max 5 rows
            if action_rows.len() == 5 {
                break;
            }
        }
    }

    let message = CreateMessage::new()
        .embed(embed)
        .components(action_rows);

    // Send the message to Discord
    match state.http.send_message(channel_id_u64.into(), Vec::new(), &message).await {
        Ok(sent_message) => {
            // Update the config with the message ID
            let mut config = config;
            if let Err(e) = config.update_message_id(&state.db, &sent_message.id.to_string()).await {
                tracing::error!("Failed to update message ID in database: {}", e);
                // The message was sent successfully, but we couldn't update the DB
                // This is not a critical error, but log it
            }

            Ok(Json(serde_json::json!({
                "success": true,
                "message": "Self-role message deployed successfully",
                "message_id": sent_message.id.to_string(),
                "config_id": config.id
            })))
        }
        Err(e) => {
            tracing::error!("Failed to send self-role message to Discord: {}", e);
            // Clean up the database entries since the message failed to send
            let _ = config.delete(&state.db).await;

            Ok(Json(serde_json::json!({
                "success": false,
                "message": format!("Failed to send message to Discord: {}", e)
            })))
        }
    }
}

async fn api_update_selfroles(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path((guild_id, config_id)): Path<(String, String)>,
    Json(payload): Json<CreateSelfRoleRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let (_, user) = extract_session_data(&headers).await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    let user = user.ok_or(StatusCode::UNAUTHORIZED)?;
    
    if !user.has_manage_roles_in_guild(&guild_id) {
        return Err(StatusCode::FORBIDDEN);
    }
    
    let config_id: i64 = config_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    
    // Validate title and body are not empty
    if payload.title.trim().is_empty() {
        return Ok(Json(serde_json::json!({
            "success": false,
            "message": "Title cannot be empty"
        })));
    }
    
    if payload.body.trim().is_empty() {
        return Ok(Json(serde_json::json!({
            "success": false,
            "message": "Body cannot be empty"
        })));
    }
    
    // Validate title and body length
    if payload.title.len() > 256 {
        return Ok(Json(serde_json::json!({
            "success": false,
            "message": "Title must be 256 characters or less"
        })));
    }
    
    if payload.body.len() > 2048 {
        return Ok(Json(serde_json::json!({
            "success": false,
            "message": "Body must be 2048 characters or less"
        })));
    }
    
    // Validate selection type
    if payload.selection_type != "radio" && payload.selection_type != "multiple" {
        return Ok(Json(serde_json::json!({
            "success": false,
            "message": "Invalid selection type. Must be 'radio' or 'multiple'"
        })));
    }
    
    // Validate roles count
    if payload.roles.is_empty() {
        return Ok(Json(serde_json::json!({
            "success": false,
            "message": "At least one role must be selected"
        })));
    }
    
    if payload.roles.len() > 25 {
        return Ok(Json(serde_json::json!({
            "success": false,
            "message": "Maximum 25 roles allowed per self-role message"
        })));
    }
    
    // Get existing config and verify it belongs to this guild
    let configs = match crate::database::selfroles::SelfRoleConfig::get_by_guild(&state.db, &guild_id).await {
        Ok(configs) => configs,
        Err(e) => {
            tracing::error!("Failed to fetch self-role configs for guild {}: {}", guild_id, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    let mut config = match configs.into_iter().find(|c| c.id == config_id) {
        Some(config) => config,
        None => return Err(StatusCode::NOT_FOUND),
    };
    
    // Update the config
    if let Err(e) = config.update(
        &state.db,
        &payload.title,
        &payload.body,
        &payload.selection_type,
    ).await {
        tracing::error!("Failed to update self-role config: {}", e);
        return Ok(Json(serde_json::json!({
            "success": false,
            "message": "Failed to update configuration"
        })));
    }
    
    // Delete existing roles for this config
    if let Err(e) = crate::database::selfroles::SelfRoleRole::delete_by_config_id(&state.db, config.id).await {
        tracing::error!("Failed to delete existing roles for config {}: {}", config.id, e);
        return Ok(Json(serde_json::json!({
            "success": false,
            "message": "Failed to update role configuration"
        })));
    }
    
    // Create new role entries
    for role_data in &payload.roles {
        if let Err(e) = crate::database::selfroles::SelfRoleRole::create(
            &state.db,
            config.id,
            &role_data.role_id,
            &role_data.emoji,
        ).await {
            tracing::error!("Failed to create self-role entry: {}", e);
            return Ok(Json(serde_json::json!({
                "success": false,
                "message": "Failed to save role configuration"
            })));
        }
    }
    
    // Update Discord message if message_id exists
    if let Some(message_id) = &config.message_id {
        let guild_id_u64: u64 = guild_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
        let channel_id_u64: u64 = config.channel_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
        let message_id_u64: u64 = message_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
        
        // Create updated Discord embed and buttons
        use serenity::all::{CreateEmbed, CreateActionRow, CreateButton, ButtonStyle, Colour, EditMessage};
        
        let embed = CreateEmbed::new()
            .title(&payload.title)
            .description(&payload.body)
            .colour(Colour::from_rgb(102, 126, 234));
        
        // Create buttons
        let mut action_rows = Vec::new();
        let mut current_row = Vec::new();
        
        for (index, role_data) in payload.roles.iter().enumerate() {
            let button = CreateButton::new(format!("selfrole_{}_{}", config.id, role_data.role_id))
                .label(&format!("{} {}", role_data.emoji, 
                    // Get role name from Discord API
                    match state.http.get_guild_roles(guild_id_u64.into()).await {
                        Ok(roles) => {
                            roles.iter()
                                .find(|r| r.id.to_string() == role_data.role_id)
                                .map(|r| r.name.clone())
                                .unwrap_or_else(|| format!("Role {}", role_data.role_id))
                        }
                        Err(_) => format!("Role {}", role_data.role_id)
                    }
                ))
                .style(ButtonStyle::Primary);
            
            current_row.push(button);
            
            if current_row.len() == 5 || index == payload.roles.len() - 1 {
                action_rows.push(CreateActionRow::Buttons(current_row.clone()));
                current_row.clear();
                
                if action_rows.len() == 5 {
                    break;
                }
            }
        }
        
        let edit_message = EditMessage::new()
            .embed(embed)
            .components(action_rows);
        
        // Update the message on Discord
        if let Err(e) = state.http.edit_message(channel_id_u64.into(), message_id_u64.into(), &edit_message, Vec::new()).await {
            tracing::error!("Failed to update self-role message on Discord: {}", e);
            // Don't fail the whole operation if Discord update fails
        }
    }
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Self-role message updated successfully",
        "config_id": config.id
    })))
}

async fn api_get_channels(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let (_, user) = extract_session_data(&headers).await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let user = user.ok_or(StatusCode::UNAUTHORIZED)?;

    if !user.has_manage_roles_in_guild(&guild_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    let guild_id_u64: u64 = guild_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.http.get_channels(guild_id_u64.into()).await {
        Ok(channels) => {
            let channel_data: Vec<serde_json::Value> = channels
                .into_iter()
                .filter(|channel| matches!(channel.kind, serenity::all::ChannelType::Text)) // Text channels only
                .map(|channel| serde_json::json!({
                    "id": channel.id.to_string(),
                    "name": channel.name,
                    "type": 0 // Text channel type
                }))
                .collect();

            Ok(Json(serde_json::json!({
                "channels": channel_data
            })))
        }
        Err(e) => {
            tracing::error!("Failed to fetch channels for guild {}: {}", guild_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn api_get_roles(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let (_, user) = extract_session_data(&headers).await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let user = user.ok_or(StatusCode::UNAUTHORIZED)?;

    if !user.has_manage_roles_in_guild(&guild_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    let guild_id_u64: u64 = guild_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.http.get_guild_roles(guild_id_u64.into()).await {
        Ok(roles) => {
            let role_data: Vec<serde_json::Value> = roles
                .into_iter()
                .filter(|role| role.name != "@everyone") // Filter out @everyone role
                .map(|role| serde_json::json!({
                    "id": role.id.to_string(),
                    "name": role.name,
                    "color": role.colour.0,
                    "position": role.position
                }))
                .collect();

            Ok(Json(serde_json::json!({
                "roles": role_data
            })))
        }
        Err(e) => {
            tracing::error!("Failed to fetch roles for guild {}: {}", guild_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn api_get_selfroles(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let (_, user) = extract_session_data(&headers).await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let user = user.ok_or(StatusCode::UNAUTHORIZED)?;

    if !user.has_manage_roles_in_guild(&guild_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    match crate::database::selfroles::SelfRoleConfig::get_by_guild(&state.db, &guild_id).await {
        Ok(configs) => {
            let mut config_data = Vec::new();

            for config in configs {
                let roles = match config.get_roles(&state.db).await {
                    Ok(roles) => roles,
                    Err(e) => {
                        tracing::error!("Failed to get roles for config {}: {}", config.id, e);
                        continue;
                    }
                };

                config_data.push(serde_json::json!({
                    "id": config.id,
                    "title": config.title,
                    "body": config.body,
                    "selection_type": config.selection_type,
                    "channel_id": config.channel_id,
                    "message_id": config.message_id,
                    "created_at": config.created_at,
                    "updated_at": config.updated_at,
                    "role_count": roles.len()
                }));
            }

            Ok(Json(serde_json::json!({
                "success": true,
                "configs": config_data
            })))
        }
        Err(e) => {
            tracing::error!("Failed to fetch self-role configs for guild {}: {}", guild_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn api_get_selfrole_config(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path((guild_id, config_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let (_, user) = extract_session_data(&headers).await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let user = user.ok_or(StatusCode::UNAUTHORIZED)?;

    if !user.has_manage_roles_in_guild(&guild_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    let config_id: i64 = config_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;

    // Get config by ID and verify it belongs to this guild
    let configs = match crate::database::selfroles::SelfRoleConfig::get_by_guild(&state.db, &guild_id).await {
        Ok(configs) => configs,
        Err(e) => {
            tracing::error!("Failed to fetch self-role configs for guild {}: {}", guild_id, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let config = match configs.into_iter().find(|c| c.id == config_id) {
        Some(config) => config,
        None => return Err(StatusCode::NOT_FOUND),
    };

    let roles = match config.get_roles(&state.db).await {
        Ok(roles) => roles,
        Err(e) => {
            tracing::error!("Failed to get roles for config {}: {}", config.id, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok(Json(serde_json::json!({
        "success": true,
        "config": {
            "id": config.id,
            "title": config.title,
            "body": config.body,
            "selection_type": config.selection_type,
            "channel_id": config.channel_id,
            "message_id": config.message_id,
            "created_at": config.created_at,
            "updated_at": config.updated_at,
            "roles": roles.iter().map(|r| serde_json::json!({
                "role_id": r.role_id,
                "emoji": r.emoji
            })).collect::<Vec<_>>()
        }
    })))
}

async fn api_delete_selfroles(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path((guild_id, config_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let (_, user) = extract_session_data(&headers).await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    let user = user.ok_or(StatusCode::UNAUTHORIZED)?;
    
    if !user.has_manage_roles_in_guild(&guild_id) {
        return Err(StatusCode::FORBIDDEN);
    }
    
    let config_id: i64 = config_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    
    // Get existing config and verify it belongs to this guild
    let configs = match crate::database::selfroles::SelfRoleConfig::get_by_guild(&state.db, &guild_id).await {
        Ok(configs) => configs,
        Err(e) => {
            tracing::error!("Failed to fetch self-role configs for guild {}: {}", guild_id, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    let config = match configs.into_iter().find(|c| c.id == config_id) {
        Some(config) => config,
        None => return Err(StatusCode::NOT_FOUND),
    };
    
    // Delete the Discord message if it exists
    if let Some(message_id) = &config.message_id {
        let channel_id_u64: u64 = config.channel_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
        let message_id_u64: u64 = message_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
        
        // Try to delete the Discord message
        if let Err(e) = state.http.delete_message(channel_id_u64.into(), message_id_u64.into(), Some("Self-role configuration deleted")).await {
            tracing::warn!("Failed to delete Discord message {}: {}", message_id, e);
            // Don't fail the operation if Discord message deletion fails
        }
    }
    
    // Delete the configuration from database (this will cascade delete roles)
    if let Err(e) = config.delete(&state.db).await {
        tracing::error!("Failed to delete self-role config: {}", e);
        return Ok(Json(serde_json::json!({
            "success": false,
            "message": "Failed to delete configuration"
        })));
    }
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Self-role message deleted successfully"
    })))
}
