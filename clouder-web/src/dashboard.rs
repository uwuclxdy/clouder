use crate::WebState;
use crate::session;
use axum::extract::{Path, Query, State};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum_extra::extract::cookie::SignedCookieJar;
use clouder_core::database::guild_cache::CachedGuild;
use clouder_core::utils::has_permission;
use serde::Deserialize;
use serenity::all::Permissions;

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

#[derive(Deserialize, Default)]
pub struct LoginQuery {
    error: Option<String>,
}

pub async fn index(jar: SignedCookieJar) -> Response {
    match session::extract(&jar) {
        Some(_) => Redirect::to("/servers").into_response(),
        None => Redirect::to("/login").into_response(),
    }
}

pub async fn login_page(jar: SignedCookieJar, Query(query): Query<LoginQuery>) -> Response {
    if session::extract(&jar).is_some() {
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
            format!(r#"<p class="error-msg">{}</p>"#, msg)
        })
        .unwrap_or_default();

    Html(render(LOGIN_HTML, &[("ERROR_MSG", &error_html)])).into_response()
}

pub async fn servers_page(State(state): State<WebState>, jar: SignedCookieJar) -> Response {
    let Some(user) = session::extract(&jar) else {
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

    Html(render(
        SERVERS_HTML,
        &[
            ("USERNAME", &user.username),
            ("AVATAR_URL", &user.avatar_url()),
            ("SERVER_CARDS", &cards),
            ("HAS_CACHE", &has_cache),
            ("GUILD_INSTALL_URL", &guild_install_url),
            ("USER_INSTALL_URL", &user_install_url),
        ],
    ))
    .into_response()
}

fn render_guild_cards(guilds: &[CachedGuild]) -> String {
    guilds
        .iter()
        .map(|g| {
            let name = html_escape(&g.name);
            let icon_html = match &g.icon {
                Some(hash) => format!(
                    r#"<img src="https://cdn.discordapp.com/icons/{}/{}.png?size=64" alt="" class="server-icon">"#,
                    g.guild_id, hash
                ),
                None => {
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
                r#"<div class="card server-card" role="button" tabindex="0" onclick="location.href='/dashboard/{}/about'" onkeydown="if(event.key==='Enter')location.href='/dashboard/{}/about'">
                    {}
                    <div class="server-info">
                        <span class="server-name">{}</span>
                        <span class="label">{}</span>
                    </div>
                    <span class="arrow">→</span>
                </div>"#,
                g.guild_id, g.guild_id, icon_html, name, g.guild_id
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub async fn dashboard_redirect(Path(guild_id): Path<String>) -> Redirect {
    Redirect::to(&format!("/dashboard/{}/about", guild_id))
}

pub async fn selfroles_page(
    State(state): State<WebState>,
    jar: SignedCookieJar,
    Path(guild_id): Path<String>,
) -> Response {
    let Some(user) = session::extract(&jar) else {
        return Redirect::to("/login").into_response();
    };
    let Some(raw_perms) = guild_perms(&state, &user.user_id, &guild_id).await else {
        return Redirect::to("/servers").into_response();
    };
    let perms = Permissions::from_bits_truncate(raw_perms as u64);
    if !has_permission(perms, Permissions::MANAGE_ROLES) {
        return Redirect::to("/servers").into_response();
    }
    let guild_name = guild_name_or_id(&state, &user.user_id, &guild_id).await;
    let sidebar = render_sidebar(&guild_id, "selfroles", raw_perms);
    Html(render(
        SELFROLES_HTML,
        &[
            ("USERNAME", &user.username),
            ("AVATAR_URL", &user.avatar_url()),
            ("GUILD_ID", &guild_id),
            ("GUILD_NAME", &guild_name),
            ("SIDEBAR_LINKS", &sidebar),
        ],
    ))
    .into_response()
}

pub async fn welcome_goodbye_page(
    State(state): State<WebState>,
    jar: SignedCookieJar,
    Path(guild_id): Path<String>,
) -> Response {
    let Some(user) = session::extract(&jar) else {
        return Redirect::to("/login").into_response();
    };
    let Some(raw_perms) = guild_perms(&state, &user.user_id, &guild_id).await else {
        return Redirect::to("/servers").into_response();
    };
    let perms = Permissions::from_bits_truncate(raw_perms as u64);
    if !has_permission(perms, Permissions::MANAGE_GUILD) {
        return Redirect::to("/servers").into_response();
    }
    let default_color = format!("#{:06X}", state.app_state.config.web.embed.default_color);
    let guild_name = guild_name_or_id(&state, &user.user_id, &guild_id).await;
    let sidebar = render_sidebar(&guild_id, "welcome-goodbye", raw_perms);
    Html(render(
        WELCOME_HTML,
        &[
            ("USERNAME", &user.username),
            ("AVATAR_URL", &user.avatar_url()),
            ("GUILD_ID", &guild_id),
            ("GUILD_NAME", &guild_name),
            ("DEFAULT_COLOR", &default_color),
            ("SIDEBAR_LINKS", &sidebar),
        ],
    ))
    .into_response()
}

pub async fn mediaonly_page(
    State(state): State<WebState>,
    jar: SignedCookieJar,
    Path(guild_id): Path<String>,
) -> Response {
    let Some(user) = session::extract(&jar) else {
        return Redirect::to("/login").into_response();
    };
    let Some(raw_perms) = guild_perms(&state, &user.user_id, &guild_id).await else {
        return Redirect::to("/servers").into_response();
    };
    let perms = Permissions::from_bits_truncate(raw_perms as u64);
    if !has_permission(perms, Permissions::MANAGE_CHANNELS) {
        return Redirect::to("/servers").into_response();
    }
    let guild_name = guild_name_or_id(&state, &user.user_id, &guild_id).await;
    let sidebar = render_sidebar(&guild_id, "mediaonly", raw_perms);
    Html(render(
        MEDIAONLY_HTML,
        &[
            ("USERNAME", &user.username),
            ("AVATAR_URL", &user.avatar_url()),
            ("GUILD_ID", &guild_id),
            ("GUILD_NAME", &guild_name),
            ("SIDEBAR_LINKS", &sidebar),
        ],
    ))
    .into_response()
}

pub async fn about_page(
    State(state): State<WebState>,
    jar: SignedCookieJar,
    Path(guild_id): Path<String>,
) -> Response {
    let Some(user) = session::extract(&jar) else {
        return Redirect::to("/login").into_response();
    };
    let Some(raw_perms) = guild_perms(&state, &user.user_id, &guild_id).await else {
        return Redirect::to("/servers").into_response();
    };
    let perms = Permissions::from_bits_truncate(raw_perms as u64);
    if !has_permission(perms, Permissions::MANAGE_GUILD) {
        return Redirect::to("/servers").into_response();
    }
    let guild_name = guild_name_or_id(&state, &user.user_id, &guild_id).await;
    let sidebar = render_sidebar(&guild_id, "about", raw_perms);
    Html(render(
        ABOUT_HTML,
        &[
            ("USERNAME", &user.username),
            ("AVATAR_URL", &user.avatar_url()),
            ("GUILD_ID", &guild_id),
            ("GUILD_NAME", &guild_name),
            ("SIDEBAR_LINKS", &sidebar),
        ],
    ))
    .into_response()
}

pub async fn uwufy_page(
    State(state): State<WebState>,
    jar: SignedCookieJar,
    Path(guild_id): Path<String>,
) -> Response {
    let Some(user) = session::extract(&jar) else {
        return Redirect::to("/login").into_response();
    };
    let Some(raw_perms) = guild_perms(&state, &user.user_id, &guild_id).await else {
        return Redirect::to("/servers").into_response();
    };
    let perms = Permissions::from_bits_truncate(raw_perms as u64);
    if !has_permission(perms, Permissions::MANAGE_GUILD) {
        return Redirect::to("/servers").into_response();
    }
    let guild_name = guild_name_or_id(&state, &user.user_id, &guild_id).await;
    let sidebar = render_sidebar(&guild_id, "uwufy", raw_perms);
    Html(render(
        UWUFY_HTML,
        &[
            ("USERNAME", &user.username),
            ("AVATAR_URL", &user.avatar_url()),
            ("GUILD_ID", &guild_id),
            ("GUILD_NAME", &guild_name),
            ("SIDEBAR_LINKS", &sidebar),
        ],
    ))
    .into_response()
}

pub async fn profile_page(State(state): State<WebState>, jar: SignedCookieJar) -> Response {
    let Some(user) = session::extract(&jar) else {
        return Redirect::to("/login").into_response();
    };

    let dashboard_user =
        match clouder_core::DashboardUser::upsert(&state.app_state.db, &user.user_id).await {
            Ok(u) => u,
            Err(e) => {
                tracing::error!("failed to upsert dashboard user: {}", e);
                return Redirect::to("/servers").into_response();
            }
        };

    let api_base = &state.app_state.config.web.api_base;

    Html(render(
        PROFILE_HTML,
        &[
            ("USERNAME", &user.username),
            ("AVATAR_URL", &user.avatar_url()),
            ("API_KEY", &dashboard_user.api_key),
            ("USER_ID", &user.user_id),
            ("API_BASE", api_base),
        ],
    ))
    .into_response()
}

pub async fn reminders_page(
    State(state): State<WebState>,
    jar: SignedCookieJar,
    Path(guild_id): Path<String>,
) -> Response {
    let Some(user) = session::extract(&jar) else {
        return Redirect::to("/login").into_response();
    };
    let Some(raw_perms) = guild_perms(&state, &user.user_id, &guild_id).await else {
        return Redirect::to("/servers").into_response();
    };
    let perms = Permissions::from_bits_truncate(raw_perms as u64);
    if !has_permission(perms, Permissions::MANAGE_GUILD) {
        return Redirect::to("/servers").into_response();
    }
    let guild_name = guild_name_or_id(&state, &user.user_id, &guild_id).await;
    let sidebar = render_sidebar(&guild_id, "reminders", raw_perms);
    Html(render(
        REMINDERS_HTML,
        &[
            ("USERNAME", &user.username),
            ("AVATAR_URL", &user.avatar_url()),
            ("GUILD_ID", &guild_id),
            ("GUILD_NAME", &guild_name),
            ("SIDEBAR_LINKS", &sidebar),
        ],
    ))
    .into_response()
}
