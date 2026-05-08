use crate::WebState;
use axum::extract::FromRef;
use axum_extra::extract::cookie::{Cookie, Key, SameSite, SignedCookieJar};
use clouder_core::database::dashboard_sessions::DashboardSession;
use cookie::time::Duration;

/// Server-backed session: the cookie holds an opaque session ID only. The
/// associated user identity, OAuth token, and CSRF token live in
/// `dashboard_sessions` and `dashboard_users` and are looked up per request.
#[derive(Debug, Clone)]
pub struct SessionUser {
    pub user_id: String,
    pub session_id: String,
    pub csrf_token: String,
}

const COOKIE_NAME: &str = "clouder_session";
pub const SESSION_TTL_SECONDS: i64 = 24 * 60 * 60;

pub async fn extract(state: &WebState, jar: &SignedCookieJar) -> Option<SessionUser> {
    let session_id = jar.get(COOKIE_NAME)?.value().to_string();
    let session = DashboardSession::get_active(&state.app_state.db, &session_id)
        .await
        .ok()
        .flatten()?;
    Some(SessionUser {
        user_id: session.user_id,
        session_id: session.session_id,
        csrf_token: session.csrf_token,
    })
}

pub fn store_cookie(jar: SignedCookieJar, session_id: &str, secure: bool) -> SignedCookieJar {
    let mut cookie = Cookie::new(COOKIE_NAME, session_id.to_string());
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_secure(secure);
    cookie.set_max_age(Duration::seconds(SESSION_TTL_SECONDS));
    jar.add(cookie)
}

pub fn read_cookie(jar: &SignedCookieJar) -> Option<String> {
    jar.get(COOKIE_NAME).map(|c| c.value().to_string())
}

pub fn clear(jar: SignedCookieJar) -> SignedCookieJar {
    let mut cookie = Cookie::from(COOKIE_NAME);
    cookie.set_path("/");
    jar.remove(cookie)
}

/// Extractor for API routes — returns 401 if session cookie is missing or invalid.
pub struct Auth(pub SessionUser);

impl<S> axum::extract::FromRequestParts<S> for Auth
where
    S: Send + Sync,
    Key: FromRef<S>,
    WebState: FromRef<S>,
{
    type Rejection = axum::http::StatusCode;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let jar = SignedCookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| axum::http::StatusCode::UNAUTHORIZED)?;
        let web_state = WebState::from_ref(state);
        extract(&web_state, &jar)
            .await
            .map(Auth)
            .ok_or(axum::http::StatusCode::UNAUTHORIZED)
    }
}

/// Extractor for state-changing API routes: requires both a valid session and
/// a matching `X-CSRF-Token` header. Defends against cross-site requests that
/// `SameSite=Lax` doesn't cover (top-level POST navigations, subdomain abuse).
pub struct CsrfAuth(pub SessionUser);

impl<S> axum::extract::FromRequestParts<S> for CsrfAuth
where
    S: Send + Sync,
    Key: FromRef<S>,
    WebState: FromRef<S>,
{
    type Rejection = axum::http::StatusCode;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let auth = Auth::from_request_parts(parts, state).await?;
        let presented = parts
            .headers
            .get("x-csrf-token")
            .and_then(|v| v.to_str().ok())
            .ok_or(axum::http::StatusCode::FORBIDDEN)?;

        // Reload the canonical CSRF token from the session store and constant-time compare.
        let web_state = WebState::from_ref(state);
        let session = clouder_core::database::dashboard_sessions::DashboardSession::get_active(
            &web_state.app_state.db,
            &auth.0.session_id,
        )
        .await
        .ok()
        .flatten()
        .ok_or(axum::http::StatusCode::UNAUTHORIZED)?;

        if !session.csrf_matches(presented) {
            return Err(axum::http::StatusCode::FORBIDDEN);
        }
        Ok(CsrfAuth(auth.0))
    }
}
