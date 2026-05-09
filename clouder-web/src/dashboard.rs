use crate::WebState;
use crate::session::{self, SessionUser};
use axum::extract::{Path, Query, State};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum_extra::extract::cookie::SignedCookieJar;
use clouder_core::DashboardUser;
use clouder_core::database::guild_cache::CachedGuild;
use clouder_core::utils::has_permission;
use serde::Deserialize;
use serenity::all::Permissions;
use tracing::error;

// Full page templates
static LOGIN_HTML: &str = include_str!("../templates/login.html");
static SERVERS_HTML: &str = include_str!("../templates/servers.html");
static SELFROLES_HTML: &str = include_str!("../templates/selfroles.html");
static WELCOME_HTML: &str = include_str!("../templates/welcome_goodbye.html");
static MEDIAONLY_HTML: &str = include_str!("../templates/mediaonly.html");
static ABOUT_HTML: &str = include_str!("../templates/about.html");
static UWUFY_HTML: &str = include_str!("../templates/uwufy.html");
static PROFILE_HTML: &str = include_str!("../templates/profile.html");
static REMINDERS_HTML: &str = include_str!("../templates/reminders.html");

async fn guild_name_or_id(state: &WebState, user_id: &str, guild_id: &str) -> String {
    CachedGuild::get_name(&state.app_state.db, user_id, guild_id)
        .await
        .unwrap_or_else(|| guild_id.to_string())
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn render(template: &str, vars: &[(&str, &str)]) -> String {
    vars.iter().fold(template.to_string(), |acc, (key, val)| {
        acc.replace(&format!("{{{{{}}}}}", key), val)
    })
}

/// Returns the cached permissions for a user in a guild, or `None` if not cached.
/// A `None` result means the user has no access; redirect to `/servers`.
async fn guild_perms(state: &WebState, user_id: &str, guild_id: &str) -> Option<i64> {
    CachedGuild::get_user_permissions(&state.app_state.db, user_id, guild_id)
        .await
        .ok()
        .flatten()
}

/// Renders the sidebar nav links, showing only pages the user has permission to access.
fn render_sidebar(guild_id: &str, active: &str, raw_perms: i64) -> String {
    let perms = Permissions::from_bits_truncate(raw_perms as u64);

    let pages: &[(&str, &str, Permissions)] = &[
        ("about", "about", Permissions::MANAGE_GUILD),
        ("selfroles", "self-roles", Permissions::MANAGE_ROLES),
        (
            "welcome-goodbye",
            "welcome/goodbye",
            Permissions::MANAGE_GUILD,
        ),
        ("reminders", "reminders", Permissions::MANAGE_GUILD),
        ("mediaonly", "media-only", Permissions::MANAGE_CHANNELS),
        ("uwufy", "uwufy", Permissions::MANAGE_GUILD),
    ];

    pages
        .iter()
        .filter(|(_, _, req)| has_permission(perms, *req))
        .map(|(path, label, _)| {
            let active_class = if *path == active { " active" } else { "" };
            format!(
                r#"<a href="/dashboard/{}/{}" class="sidebar-link{}">{}</a>"#,
                guild_id, path, active_class, label
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Validates the guild_id is a numeric Discord snowflake. Rejecting non-numeric
/// input here removes any chance of HTML/JS injection through `guild_id` placeholders
/// in the templates and prevents path-traversal-style abuse downstream.
fn parse_snowflake(s: &str) -> Option<u64> {
    s.parse::<u64>().ok()
}

#[derive(Debug, Clone)]
struct DisplayProfile {
    username: String,
    avatar_url: String,
}

/// Discord avatar hashes are 32-char hex (static) or `a_` + 32 hex (animated).
/// Anything else is malformed input — possibly an injection attempt — so fall
/// back to a default avatar rather than splicing the value into a URL.
fn is_valid_avatar_hash(hash: &str) -> bool {
    let body = hash.strip_prefix("a_").unwrap_or(hash);
    !body.is_empty() && body.chars().all(|c| c.is_ascii_hexdigit())
}

fn avatar_url_for(user_id: &str, avatar: Option<&str>) -> String {
    match avatar {
        Some(hash) if is_valid_avatar_hash(hash) => format!(
            "https://cdn.discordapp.com/avatars/{}/{}.png?size=64",
            user_id, hash
        ),
        _ => {
            let index = user_id.parse::<u64>().unwrap_or(0) % 6;
            format!("https://cdn.discordapp.com/embed/avatars/{}.png", index)
        }
    }
}

/// Loads the cached profile for a session user. Returns escaped strings safe
/// for direct insertion into HTML templates.
async fn load_profile(state: &WebState, user: &SessionUser) -> DisplayProfile {
    let row = DashboardUser::get_by_user_id(&state.app_state.db, &user.user_id)
        .await
        .ok()
        .flatten();
    let username = row
        .as_ref()
        .and_then(|u| u.username.clone())
        .unwrap_or_else(|| "user".to_string());
    let avatar = row.as_ref().and_then(|u| u.avatar.clone());
    DisplayProfile {
        username: html_escape(&username),
        avatar_url: html_escape(&avatar_url_for(&user.user_id, avatar.as_deref())),
    }
}

#[derive(Deserialize, Default)]
pub struct LoginQuery {
    error: Option<String>,
}

pub async fn index(State(state): State<WebState>, jar: SignedCookieJar) -> Response {
    match session::extract(&state, &jar).await {
        Some(_) => Redirect::to("/servers").into_response(),
        None => Redirect::to("/login").into_response(),
    }
}

pub async fn login_page(
    State(state): State<WebState>,
    jar: SignedCookieJar,
    Query(query): Query<LoginQuery>,
) -> Response {
    if session::extract(&state, &jar).await.is_some() {
        return Redirect::to("/servers").into_response();
    }

    let error_html = query
        .error
        .as_deref()
        .map(|e| {
            let msg = match e {
                "denied" => "access denied.",
                "auth_failed" => "authentication failed. try again.",
                "missing_code" => "invalid oauth response.",
                _ => "something went wrong.",
            };
            format!(r#"<p class="error-msg">{}</p>"#, html_escape(msg))
        })
        .unwrap_or_default();

    Html(render(LOGIN_HTML, &[("ERROR_MSG", &error_html)])).into_response()
}

pub async fn servers_page(State(state): State<WebState>, jar: SignedCookieJar) -> Response {
    let Some(user) = session::extract(&state, &jar).await else {
        return Redirect::to("/login").into_response();
    };

    let cached_guilds = CachedGuild::get_for_user(&state.app_state.db, &user.user_id)
        .await
        .unwrap_or_default();

    let redirect_uri = urlencoding::encode(&state.app_state.config.web.oauth.redirect_uri);
    let client_id = &state.app_state.config.web.oauth.client_id;
    let guild_install_url = format!(
        "https://discord.com/oauth2/authorize?client_id={}&permissions=8&response_type=code&redirect_uri={}&integration_type=0&scope=bot",
        client_id, redirect_uri,
    );
    let user_install_url = format!(
        "https://discord.com/oauth2/authorize?client_id={}&response_type=code&redirect_uri={}&integration_type=1&scope=applications.commands",
        client_id, redirect_uri,
    );

    let cards = render_guild_cards(&cached_guilds);
    let has_cache = (!cached_guilds.is_empty()).to_string();
    let profile = load_profile(&state, &user).await;
    let csrf = html_escape(&user.csrf_token);

    Html(render(
        SERVERS_HTML,
        &[
            ("USERNAME", &profile.username),
            ("AVATAR_URL", &profile.avatar_url),
            ("SERVER_CARDS", &cards),
            ("HAS_CACHE", &has_cache),
            ("GUILD_INSTALL_URL", &html_escape(&guild_install_url)),
            ("USER_INSTALL_URL", &html_escape(&user_install_url)),
            ("CSRF_TOKEN", &csrf),
        ],
    ))
    .into_response()
}

fn render_guild_cards(guilds: &[CachedGuild]) -> String {
    guilds
        .iter()
        .filter(|g| parse_snowflake(&g.guild_id).is_some())
        .map(|g| {
            let name = html_escape(&g.name);
            let safe_guild_id = html_escape(&g.guild_id);
            let icon_html = match &g.icon {
                Some(hash) if is_valid_avatar_hash(hash) => format!(
                    r#"<img src="https://cdn.discordapp.com/icons/{}/{}.png?size=64" alt="" class="server-icon">"#,
                    safe_guild_id, hash
                ),
                _ => {
                    let first = g
                        .name
                        .chars()
                        .next()
                        .unwrap_or('?')
                        .to_uppercase()
                        .next()
                        .unwrap_or('?');
                    format!(
                        r#"<div class="server-icon-placeholder">{}</div>"#,
                        html_escape(&first.to_string())
                    )
                }
            };
            format!(
                r#"<div class="card server-card" role="button" tabindex="0" onclick="location.href='/dashboard/{gid}/about'" onkeydown="if(event.key==='Enter')location.href='/dashboard/{gid}/about'">
                    {icon}
                    <div class="server-info">
                        <span class="server-name">{name}</span>
                        <span class="label">{gid}</span>
                    </div>
                    <span class="arrow">→</span>
                </div>"#,
                gid = safe_guild_id,
                icon = icon_html,
                name = name,
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub async fn dashboard_redirect(Path(guild_id): Path<String>) -> Response {
    if parse_snowflake(&guild_id).is_none() {
        return Redirect::to("/servers").into_response();
    }
    Redirect::to(&format!("/dashboard/{}/about", guild_id)).into_response()
}

/// Common scaffolding for permission-gated dashboard pages: extracts the
/// session, validates the guild_id is a snowflake, checks cached permissions,
/// then returns the data the per-page handler needs to fill its template.
struct PageContext {
    profile: DisplayProfile,
    guild_id: String,
    guild_name: String,
    sidebar: String,
    csrf: String,
}

async fn page_context(
    state: &WebState,
    jar: &SignedCookieJar,
    raw_guild_id: &str,
    active: &str,
    required: Permissions,
) -> Result<PageContext, Response> {
    let Some(user) = session::extract(state, jar).await else {
        return Err(Redirect::to("/login").into_response());
    };
    if parse_snowflake(raw_guild_id).is_none() {
        return Err(Redirect::to("/servers").into_response());
    }
    let Some(raw_perms) = guild_perms(state, &user.user_id, raw_guild_id).await else {
        return Err(Redirect::to("/servers").into_response());
    };
    let perms = Permissions::from_bits_truncate(raw_perms as u64);
    if !has_permission(perms, required) {
        return Err(Redirect::to("/servers").into_response());
    }
    let guild_name = guild_name_or_id(state, &user.user_id, raw_guild_id).await;
    let sidebar = render_sidebar(raw_guild_id, active, raw_perms);
    let profile = load_profile(state, &user).await;
    let csrf = html_escape(&user.csrf_token);
    Ok(PageContext {
        profile,
        guild_id: raw_guild_id.to_string(),
        guild_name: html_escape(&guild_name),
        sidebar,
        csrf,
    })
}

pub async fn selfroles_page(
    State(state): State<WebState>,
    jar: SignedCookieJar,
    Path(guild_id): Path<String>,
) -> Response {
    let ctx = match page_context(
        &state,
        &jar,
        &guild_id,
        "selfroles",
        Permissions::MANAGE_ROLES,
    )
    .await
    {
        Ok(c) => c,
        Err(r) => return r,
    };
    Html(render(
        SELFROLES_HTML,
        &[
            ("USERNAME", &ctx.profile.username),
            ("AVATAR_URL", &ctx.profile.avatar_url),
            ("GUILD_ID", &ctx.guild_id),
            ("GUILD_NAME", &ctx.guild_name),
            ("SIDEBAR_LINKS", &ctx.sidebar),
            ("CSRF_TOKEN", &ctx.csrf),
        ],
    ))
    .into_response()
}

pub async fn welcome_goodbye_page(
    State(state): State<WebState>,
    jar: SignedCookieJar,
    Path(guild_id): Path<String>,
) -> Response {
    let ctx = match page_context(
        &state,
        &jar,
        &guild_id,
        "welcome-goodbye",
        Permissions::MANAGE_GUILD,
    )
    .await
    {
        Ok(c) => c,
        Err(r) => return r,
    };
    let default_color = format!("#{:06X}", state.app_state.config.web.embed.default_color);
    Html(render(
        WELCOME_HTML,
        &[
            ("USERNAME", &ctx.profile.username),
            ("AVATAR_URL", &ctx.profile.avatar_url),
            ("GUILD_ID", &ctx.guild_id),
            ("GUILD_NAME", &ctx.guild_name),
            ("DEFAULT_COLOR", &default_color),
            ("SIDEBAR_LINKS", &ctx.sidebar),
            ("CSRF_TOKEN", &ctx.csrf),
        ],
    ))
    .into_response()
}

pub async fn mediaonly_page(
    State(state): State<WebState>,
    jar: SignedCookieJar,
    Path(guild_id): Path<String>,
) -> Response {
    let ctx = match page_context(
        &state,
        &jar,
        &guild_id,
        "mediaonly",
        Permissions::MANAGE_CHANNELS,
    )
    .await
    {
        Ok(c) => c,
        Err(r) => return r,
    };
    Html(render(
        MEDIAONLY_HTML,
        &[
            ("USERNAME", &ctx.profile.username),
            ("AVATAR_URL", &ctx.profile.avatar_url),
            ("GUILD_ID", &ctx.guild_id),
            ("GUILD_NAME", &ctx.guild_name),
            ("SIDEBAR_LINKS", &ctx.sidebar),
            ("CSRF_TOKEN", &ctx.csrf),
        ],
    ))
    .into_response()
}

pub async fn about_page(
    State(state): State<WebState>,
    jar: SignedCookieJar,
    Path(guild_id): Path<String>,
) -> Response {
    let ctx = match page_context(&state, &jar, &guild_id, "about", Permissions::MANAGE_GUILD).await
    {
        Ok(c) => c,
        Err(r) => return r,
    };
    Html(render(
        ABOUT_HTML,
        &[
            ("USERNAME", &ctx.profile.username),
            ("AVATAR_URL", &ctx.profile.avatar_url),
            ("GUILD_ID", &ctx.guild_id),
            ("GUILD_NAME", &ctx.guild_name),
            ("SIDEBAR_LINKS", &ctx.sidebar),
            ("CSRF_TOKEN", &ctx.csrf),
        ],
    ))
    .into_response()
}

pub async fn uwufy_page(
    State(state): State<WebState>,
    jar: SignedCookieJar,
    Path(guild_id): Path<String>,
) -> Response {
    let ctx = match page_context(&state, &jar, &guild_id, "uwufy", Permissions::MANAGE_GUILD).await
    {
        Ok(c) => c,
        Err(r) => return r,
    };
    Html(render(
        UWUFY_HTML,
        &[
            ("USERNAME", &ctx.profile.username),
            ("AVATAR_URL", &ctx.profile.avatar_url),
            ("GUILD_ID", &ctx.guild_id),
            ("GUILD_NAME", &ctx.guild_name),
            ("SIDEBAR_LINKS", &ctx.sidebar),
            ("CSRF_TOKEN", &ctx.csrf),
        ],
    ))
    .into_response()
}

pub async fn profile_page(State(state): State<WebState>, jar: SignedCookieJar) -> Response {
    let Some(user) = session::extract(&state, &jar).await else {
        return Redirect::to("/login").into_response();
    };

    let dashboard_user = match DashboardUser::upsert(
        &state.app_state.db,
        &user.user_id,
        &state.app_state.config.web.api_key_pepper,
        &state.app_state.config.web.oauth_encryption_key_bytes,
    )
    .await
    {
        Ok((u, _)) => u,
        Err(e) => {
            error!("failed to upsert dashboard user: {}", e);
            return Redirect::to("/servers").into_response();
        }
    };

    let api_base = &state.app_state.config.web.api_base;
    let profile = load_profile(&state, &user).await;

    // Decrypt the stored ciphertext for display. Legacy rows (pre-migration 013)
    // only have a hash, so the user must regenerate before they can view a key.
    let decrypted = dashboard_user
        .decrypt_api_key(&state.app_state.config.web.oauth_encryption_key_bytes)
        .unwrap_or_else(|e| {
            error!("failed to decrypt api key: {}", e);
            None
        });
    let api_key_display = match decrypted.as_deref() {
        Some(k) => k,
        None if dashboard_user.api_key_hash.is_some() => "(hidden — regenerate to view a new key)",
        None => "(no key — click regenerate)",
    };

    Html(render(
        PROFILE_HTML,
        &[
            ("USERNAME", &profile.username),
            ("AVATAR_URL", &profile.avatar_url),
            ("API_KEY", &html_escape(api_key_display)),
            ("USER_ID", &html_escape(&user.user_id)),
            ("API_BASE", &html_escape(api_base)),
            ("CSRF_TOKEN", &html_escape(&user.csrf_token)),
        ],
    ))
    .into_response()
}

pub async fn reminders_page(
    State(state): State<WebState>,
    jar: SignedCookieJar,
    Path(guild_id): Path<String>,
) -> Response {
    let ctx = match page_context(
        &state,
        &jar,
        &guild_id,
        "reminders",
        Permissions::MANAGE_GUILD,
    )
    .await
    {
        Ok(c) => c,
        Err(r) => return r,
    };
    Html(render(
        REMINDERS_HTML,
        &[
            ("USERNAME", &ctx.profile.username),
            ("AVATAR_URL", &ctx.profile.avatar_url),
            ("GUILD_ID", &ctx.guild_id),
            ("GUILD_NAME", &ctx.guild_name),
            ("SIDEBAR_LINKS", &ctx.sidebar),
            ("CSRF_TOKEN", &ctx.csrf),
        ],
    ))
    .into_response()
}
