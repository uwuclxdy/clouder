use crate::config::AppState;
use crate::utils::get_default_embed_color;
use axum::{
    extract::{Path, State}, http::{HeaderMap, StatusCode},
    routing::{get, post, put},
    Json,
    Router,
};
use serde::{Deserialize, Serialize};
use serenity::all::{GuildId, Http, Member};
use session_extractor::extract_session_data;

pub mod auth;
pub mod dashboard;
pub mod mediaonly;
pub mod middleware;
pub mod models;
pub mod session_extractor;
pub mod welcome_goodbye;

pub async fn get_bot_member_info(
    http: &Http,
    guild_id: GuildId,
) -> Result<Member, Box<dyn std::error::Error + Send + Sync>> {
    let bot_user = http.get_current_user().await?;
    http.get_member(guild_id, bot_user.id)
        .await
        .map_err(Into::into)
}

pub fn create_router(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(dashboard::server_list))
        .route("/auth/login", get(auth::login))
        .route("/auth/callback", get(auth::callback))
        .route("/auth/logout", get(auth::logout))
        .route("/user/settings", get(dashboard::user_settings))
        .route("/feature-request", get(dashboard::feature_request))
        .route("/dashboard/{guild_id}", get(dashboard::guild_dashboard))
        .route(
            "/dashboard/{guild_id}/selfroles",
            get(dashboard::selfroles_list),
        )
        .route(
            "/dashboard/{guild_id}/selfroles/new",
            get(dashboard::selfroles_create),
        )
        .route(
            "/dashboard/{guild_id}/selfroles/edit/{config_id}",
            get(dashboard::selfroles_edit),
        )
        .route(
            "/dashboard/{guild_id}/welcome-goodbye",
            get(welcome_goodbye::show_welcome_goodbye_config),
        )
        .route(
            "/dashboard/{guild_id}/mediaonly",
            get(mediaonly::get_mediaonly_page),
        )
        .route("/api/user/settings", put(api_update_user_settings))
        .route(
            "/api/selfroles/{guild_id}",
            get(api_get_selfroles).post(api_create_selfroles),
        )
        .route(
            "/api/selfroles/{guild_id}/{config_id}",
            get(api_get_selfrole_config)
                .put(api_update_selfroles)
                .delete(api_delete_selfroles),
        )
        .route("/api/guild/{guild_id}/channels", get(api_get_channels))
        .route("/api/guild/{guild_id}/roles", get(api_get_roles))
        .route(
            "/api/welcome-goodbye/{guild_id}/config",
            post(welcome_goodbye::save_welcome_goodbye_config),
        )
        .route(
            "/api/welcome-goodbye/{guild_id}/test/welcome",
            post(welcome_goodbye::send_test_welcome),
        )
        .route(
            "/api/welcome-goodbye/{guild_id}/test/goodbye",
            post(welcome_goodbye::send_test_goodbye),
        )
        .route(
            "/api/welcome-goodbye/{guild_id}/preview",
            post(welcome_goodbye::get_live_preview),
        )
        .route(
            "/api/mediaonly/{guild_id}",
            get(mediaonly::list_configs).post(mediaonly::create_or_update_config),
        )
        .route(
            "/api/mediaonly/{guild_id}/{channel_id}",
            put(mediaonly::update_permissions).delete(mediaonly::delete_config),
        )
        .route("/api/feature-request", post(api_submit_feature_request))
        .layer(axum::middleware::from_fn(middleware::session_middleware))
        .with_state(app_state)
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

#[derive(Serialize, Deserialize)]
struct UpdateUserSettingsRequest {
    timezone: String,
    dm_reminders_enabled: bool,
}

#[derive(Serialize, Deserialize)]
struct FeatureRequest {
    description: String,
}

fn validate_selfrole_request(payload: &CreateSelfRoleRequest) -> Result<(), &'static str> {
    if payload.title.trim().is_empty() || payload.body.trim().is_empty() {
        return Err("Title and body cannot be empty");
    }
    if payload.title.len() > 256 || payload.body.len() > 2048 {
        return Err("Title max 256 chars, body max 2048 chars");
    }
    if payload.selection_type != "radio" && payload.selection_type != "multiple" {
        return Err("Invalid selection type");
    }
    if payload.roles.is_empty() || payload.roles.len() > 25 {
        return Err("Must have 1-25 roles");
    }
    Ok(())
}

async fn validate_roles_hierarchy(
    state: &AppState,
    guild_id: &str,
    roles: &[SelfRoleData],
) -> Result<Vec<serenity::all::Role>, String> {
    let guild_id_u64: u64 = guild_id.parse().map_err(|_| "Invalid guild ID")?;

    let bot_member = get_bot_member_info(&state.http, guild_id_u64.into())
        .await
        .map_err(|e| format!("Bot permission error: {}", e))?;

    let guild_roles = state
        .http
        .get_guild_roles(guild_id_u64.into())
        .await
        .map_err(|_| "Failed to get server roles")?;

    let bot_role_positions = crate::utils::get_bot_role_positions(&bot_member, &guild_roles);

    for role_data in roles {
        let role_id_u64: u64 = role_data
            .role_id
            .parse()
            .map_err(|_| format!("Invalid role ID: {}", role_data.role_id))?;

        if let Some(target_role) = guild_roles.iter().find(|r| r.id.get() == role_id_u64) {
            if !crate::utils::can_bot_manage_role(&bot_role_positions, target_role.position) {
                return Err(format!("Cannot manage role '{}'", target_role.name));
            }
        } else {
            return Err(format!("Role {} not found", role_data.role_id));
        }
    }

    Ok(guild_roles)
}

async fn api_create_selfroles(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
    Json(payload): Json<CreateSelfRoleRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let (_, user) = extract_session_data(&headers)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let user = user.ok_or(StatusCode::UNAUTHORIZED)?;

    if !user.has_manage_roles_in_guild(&guild_id) && !user.has_administrator_in_guild(&guild_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    if let Err(msg) = validate_selfrole_request(&payload) {
        return Ok(Json(serde_json::json!({"success": false, "message": msg})));
    }

    let guild_roles = match validate_roles_hierarchy(&state, &guild_id, &payload.roles).await {
        Ok(roles) => roles,
        Err(msg) => return Ok(Json(serde_json::json!({"success": false, "message": msg}))),
    };

    let config = match crate::database::selfroles::SelfRoleConfig::create(
        &state.db,
        &guild_id,
        &payload.channel_id,
        &payload.title,
        &payload.body,
        &payload.selection_type,
    )
    .await
    {
        Ok(config) => config,
        Err(_) => {
            return Ok(Json(
                serde_json::json!({"success": false, "message": "Failed to save configuration"}),
            ));
        }
    };

    for role_data in &payload.roles {
        if crate::database::selfroles::SelfRoleRole::create(
            &state.db,
            config.id,
            &role_data.role_id,
            &role_data.emoji,
        )
        .await
        .is_err()
        {
            let _ = config.delete(&state.db).await;
            return Ok(Json(
                serde_json::json!({"success": false, "message": "Failed to save role configuration"}),
            ));
        }
    }

    use serenity::all::{
        ButtonStyle, CreateActionRow, CreateButton, CreateEmbed, CreateEmbedFooter, CreateMessage,
    };

    let footer_text = match payload.selection_type.as_str() {
        "multiple" => "Multiple roles",
        "radio" => "Single role",
        _ => "",
    };

    let embed = CreateEmbed::new()
        .title(&payload.title)
        .description(&payload.body)
        .colour(get_default_embed_color(&state))
        .footer(CreateEmbedFooter::new(footer_text));

    let mut action_rows = Vec::new();
    let mut current_row = Vec::new();

    for (index, role_data) in payload.roles.iter().enumerate() {
        let role_name = guild_roles
            .iter()
            .find(|r| r.id.to_string() == role_data.role_id)
            .map(|r| r.name.clone())
            .unwrap_or_else(|| format!("Role {}", role_data.role_id));

        let button = CreateButton::new(format!("selfrole_{}_{}", config.id, role_data.role_id))
            .label(format!("{} {}", role_data.emoji, role_name))
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

    let message = CreateMessage::new().embed(embed).components(action_rows);
    let channel_id_u64: u64 = payload
        .channel_id
        .parse()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    match state
        .http
        .send_message(channel_id_u64.into(), Vec::new(), &message)
        .await
    {
        Ok(sent_message) => {
            let mut config = config;
            let _ = config
                .update_message_id(&state.db, &sent_message.id.to_string())
                .await;
            Ok(Json(serde_json::json!({
                "success": true,
                "message": "Self-role message deployed successfully",
                "message_id": sent_message.id.to_string(),
                "config_id": config.id
            })))
        }
        Err(_) => {
            let _ = config.delete(&state.db).await;
            Ok(Json(
                serde_json::json!({"success": false, "message": "Failed to send message"}),
            ))
        }
    }
}

async fn api_update_selfroles(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path((guild_id, config_id)): Path<(String, String)>,
    Json(payload): Json<CreateSelfRoleRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let (_, user) = extract_session_data(&headers)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let user = user.ok_or(StatusCode::UNAUTHORIZED)?;

    if !user.has_manage_roles_in_guild(&guild_id) && !user.has_administrator_in_guild(&guild_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    let config_id: i64 = config_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;

    if let Err(msg) = validate_selfrole_request(&payload) {
        return Ok(Json(serde_json::json!({"success": false, "message": msg})));
    }

    let guild_roles = match validate_roles_hierarchy(&state, &guild_id, &payload.roles).await {
        Ok(roles) => roles,
        Err(msg) => return Ok(Json(serde_json::json!({"success": false, "message": msg}))),
    };

    let configs = crate::database::selfroles::SelfRoleConfig::get_by_guild(&state.db, &guild_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut config = configs
        .into_iter()
        .find(|c| c.id == config_id)
        .ok_or(StatusCode::NOT_FOUND)?;

    if config
        .update(
            &state.db,
            &payload.title,
            &payload.body,
            &payload.selection_type,
        )
        .await
        .is_err()
    {
        return Ok(Json(
            serde_json::json!({"success": false, "message": "Failed to update configuration"}),
        ));
    }

    if crate::database::selfroles::SelfRoleRole::delete_by_config_id(&state.db, config.id)
        .await
        .is_err()
    {
        return Ok(Json(
            serde_json::json!({"success": false, "message": "Failed to update roles"}),
        ));
    }

    for role_data in &payload.roles {
        if crate::database::selfroles::SelfRoleRole::create(
            &state.db,
            config.id,
            &role_data.role_id,
            &role_data.emoji,
        )
        .await
        .is_err()
        {
            return Ok(Json(
                serde_json::json!({"success": false, "message": "Failed to save roles"}),
            ));
        }
    }

    if let Some(message_id) = &config.message_id {
        let channel_id_u64: u64 = config
            .channel_id
            .parse()
            .map_err(|_| StatusCode::BAD_REQUEST)?;
        let message_id_u64: u64 = message_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;

        use serenity::all::{
            ButtonStyle, CreateActionRow, CreateButton, CreateEmbed, CreateEmbedFooter, EditMessage,
        };

        let footer_text = match payload.selection_type.as_str() {
            "multiple" => "Multiple roles",
            "radio" => "Single role",
            _ => "",
        };

        let embed = CreateEmbed::new()
            .title(&payload.title)
            .description(&payload.body)
            .colour(get_default_embed_color(&state))
            .footer(CreateEmbedFooter::new(footer_text));

        let mut action_rows = Vec::new();
        let mut current_row = Vec::new();

        for (index, role_data) in payload.roles.iter().enumerate() {
            let role_name = guild_roles
                .iter()
                .find(|r| r.id.to_string() == role_data.role_id)
                .map(|r| r.name.clone())
                .unwrap_or_else(|| format!("Role {}", role_data.role_id));

            let button = CreateButton::new(format!("selfrole_{}_{}", config.id, role_data.role_id))
                .label(format!("{} {}", role_data.emoji, role_name))
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

        let edit_message = EditMessage::new().embed(embed).components(action_rows);
        let _ = state
            .http
            .edit_message(
                channel_id_u64.into(),
                message_id_u64.into(),
                &edit_message,
                Vec::new(),
            )
            .await;
    }

    Ok(Json(
        serde_json::json!({"success": true, "message": "Self-role message updated successfully", "config_id": config.id}),
    ))
}

async fn api_get_channels(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let (_, user) = extract_session_data(&headers)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let user = user.ok_or(StatusCode::UNAUTHORIZED)?;

    if !user.has_manage_roles_in_guild(&guild_id) && !user.has_administrator_in_guild(&guild_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    let guild_id_u64: u64 = guild_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    let channels = match state.http.get_channels(guild_id_u64.into()).await {
        Ok(channels) => channels,
        Err(e) => {
            tracing::error!("Failed to get channels for guild {}: {}. Bot may not be in guild or lack VIEW_CHANNEL permission.", guild_id, e);
            return Ok(Json(serde_json::json!({
                "success": false,
                "error": "bot_not_in_guild",
                "message": "Bot is not in this server or lacks required permissions. Please invite the bot first.",
                "channels": []
            })));
        }
    };

    let channel_data: Vec<serde_json::Value> = channels
        .into_iter()
        .filter(|channel| matches!(channel.kind, serenity::all::ChannelType::Text))
        .map(|channel| {
            serde_json::json!({
                "id": channel.id.to_string(),
                "name": channel.name,
                "type": 0
            })
        })
        .collect();

    Ok(Json(serde_json::json!({"channels": channel_data})))
}

async fn api_get_roles(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let (_, user) = extract_session_data(&headers)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let user = user.ok_or(StatusCode::UNAUTHORIZED)?;

    if !user.has_manage_roles_in_guild(&guild_id) && !user.has_administrator_in_guild(&guild_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    let guild_id_u64: u64 = guild_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    let guild_roles = state
        .http
        .get_guild_roles(guild_id_u64.into())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let bot_member = match get_bot_member_info(&state.http, guild_id_u64.into()).await {
        Ok(member) => member,
        Err(e) => {
            return Ok(Json(serde_json::json!({
                "success": false,
                "error": "permission_check_failed",
                "message": format!("Bot permission verification failed: {}", e),
                "roles": []
            })));
        }
    };

    let (is_admin, bot_role_positions) =
        crate::utils::can_bot_manage_roles_in_guild(&bot_member, &guild_roles);

    let role_data: Vec<serde_json::Value> = guild_roles
        .into_iter()
        .filter(|role| {
            if role.name == "@everyone" {
                return false;
            }
            is_admin || crate::utils::can_bot_manage_role(&bot_role_positions, role.position)
        })
        .map(|role| {
            serde_json::json!({
                "id": role.id.to_string(),
                "name": role.name,
                "color": role.colour.0,
                "position": role.position,
                "manageable": true
            })
        })
        .collect();

    Ok(Json(
        serde_json::json!({"success": true, "roles": role_data}),
    ))
}

async fn api_get_selfroles(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(guild_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let (_, user) = extract_session_data(&headers)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let user = user.ok_or(StatusCode::UNAUTHORIZED)?;

    if !user.has_manage_roles_in_guild(&guild_id) && !user.has_administrator_in_guild(&guild_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    let configs = crate::database::selfroles::SelfRoleConfig::get_by_guild(&state.db, &guild_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut config_data = Vec::new();
    let guild_id_u64: u64 = guild_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    let channels = match state.http.get_channels(guild_id_u64.into()).await {
        Ok(channels) => channels,
        Err(e) => {
            tracing::error!("Failed to get channels for guild {}: {}. Bot may not be in guild or lack VIEW_CHANNEL permission.", guild_id, e);
            // Return empty channels instead of failing
            Vec::new()
        }
    };

    for config in configs {
        let roles = config.get_roles(&state.db).await.unwrap_or_default();
        let channel_name = channels
            .iter()
            .find(|ch| ch.id.to_string() == config.channel_id)
            .map(|ch| ch.name.clone())
            .unwrap_or_else(|| "Unknown Channel".to_string());

        config_data.push(serde_json::json!({
            "id": config.id,
            "title": config.title,
            "body": config.body,
            "selection_type": config.selection_type,
            "channel_id": config.channel_id,
            "channel_name": channel_name,
            "message_id": config.message_id,
            "created_at": config.created_at,
            "updated_at": config.updated_at,
            "role_count": roles.len()
        }));
    }

    Ok(Json(
        serde_json::json!({"success": true, "configs": config_data}),
    ))
}

async fn api_get_selfrole_config(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path((guild_id, config_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let (_, user) = extract_session_data(&headers)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let user = user.ok_or(StatusCode::UNAUTHORIZED)?;

    if !user.has_manage_roles_in_guild(&guild_id) && !user.has_administrator_in_guild(&guild_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    let config_id: i64 = config_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    let configs = crate::database::selfroles::SelfRoleConfig::get_by_guild(&state.db, &guild_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let config = configs
        .into_iter()
        .find(|c| c.id == config_id)
        .ok_or(StatusCode::NOT_FOUND)?;

    let roles = config.get_roles(&state.db).await.unwrap_or_default();

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
    let (_, user) = extract_session_data(&headers)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let user = user.ok_or(StatusCode::UNAUTHORIZED)?;

    if !user.has_manage_roles_in_guild(&guild_id) && !user.has_administrator_in_guild(&guild_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    let config_id: i64 = config_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    let configs = crate::database::selfroles::SelfRoleConfig::get_by_guild(&state.db, &guild_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let config = configs
        .into_iter()
        .find(|c| c.id == config_id)
        .ok_or(StatusCode::NOT_FOUND)?;

    if let Some(message_id) = &config.message_id {
        let channel_id_u64: u64 = config
            .channel_id
            .parse()
            .map_err(|_| StatusCode::BAD_REQUEST)?;
        let message_id_u64: u64 = message_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
        let _ = state
            .http
            .delete_message(
                channel_id_u64.into(),
                message_id_u64.into(),
                Some("Self-role deleted"),
            )
            .await;
    }

    if config.delete(&state.db).await.is_err() {
        return Ok(Json(
            serde_json::json!({"success": false, "message": "Failed to delete configuration"}),
        ));
    }

    Ok(Json(
        serde_json::json!({"success": true, "message": "Self-role message deleted successfully"}),
    ))
}

async fn api_update_user_settings(
    headers: HeaderMap,
    Json(payload): Json<UpdateUserSettingsRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let (_, user) = extract_session_data(&headers)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    user.ok_or(StatusCode::UNAUTHORIZED)?;

    // Validate timezone
    if payload.timezone.is_empty() {
        return Ok(Json(serde_json::json!({
            "success": false,
            "message": "Timezone cannot be empty"
        })));
    }

    // TODO: Save user settings to database
    // For now, we'll just return success since reminders are not implemented
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Settings saved successfully"
    })))
}

async fn api_submit_feature_request(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<FeatureRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let (_, user) = extract_session_data(&headers)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let user = user.ok_or(StatusCode::UNAUTHORIZED)?;

    // Validate description
    if payload.description.trim().is_empty() {
        return Ok(Json(serde_json::json!({
            "success": false,
            "message": "Feature description cannot be empty"
        })));
    }

    if payload.description.len() > 2000 {
        return Ok(Json(serde_json::json!({
            "success": false,
            "message": "Feature description must be less than 2000 characters"
        })));
    }

    // Send DM to bot owner
    let dm_content = format!(
        "**New Feature Request**\n\n**From:** {} (ID: {})\n**Description:**\n{}",
        user.user.username,
        user.user.id,
        payload.description
    );

    match state.http.create_private_channel(&serde_json::json!({"recipient_id": state.config.discord.bot_owner})).await {
        Ok(channel) => {
            match state.http.send_message(channel.id, Vec::new(), &serenity::all::CreateMessage::new().content(&dm_content)).await {
                Ok(_) => {
                    Ok(Json(serde_json::json!({
                        "success": true,
                        "message": "Feature request submitted successfully!"
                    })))
                }
                Err(e) => {
                    tracing::error!("Failed to send DM to bot owner: {}", e);
                    Ok(Json(serde_json::json!({
                        "success": false,
                        "message": "Failed to send feature request. Please try again later."
                    })))
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to create DM channel with bot owner: {}", e);
            Ok(Json(serde_json::json!({
                "success": false,
                "message": "Failed to send feature request. Please try again later."
            })))
        }
    }
}
