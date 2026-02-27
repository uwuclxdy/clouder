use axum_extra::extract::cookie::{Cookie, Key, SameSite, SignedCookieJar};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionUser {
    pub user_id: String,
    pub username: String,
    pub avatar: Option<String>,
    pub access_token: String,
}

impl SessionUser {
    pub fn avatar_url(&self) -> String {
        match &self.avatar {
            Some(hash) => format!(
                "https://cdn.discordapp.com/avatars/{}/{}.png?size=64",
                self.user_id, hash
            ),
            None => {
                let index = self.user_id.parse::<u64>().unwrap_or(0) % 6;
                format!("https://cdn.discordapp.com/embed/avatars/{}.png", index)
            }
        }
    }
}

const COOKIE_NAME: &str = "clouder_session";

pub fn extract(jar: &SignedCookieJar) -> Option<SessionUser> {
    serde_json::from_str(jar.get(COOKIE_NAME)?.value()).ok()
}

pub fn store(jar: SignedCookieJar, user: &SessionUser, secure: bool) -> SignedCookieJar {
    let value = serde_json::to_string(user).expect("session serialization");
    let mut cookie = Cookie::new(COOKIE_NAME, value);
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_secure(secure);
    jar.add(cookie)
}

pub fn clear(jar: SignedCookieJar) -> SignedCookieJar {
    jar.remove(Cookie::from(COOKIE_NAME))
}

/// Extractor for API routes — returns 401 if session cookie is missing or invalid.
pub struct Auth(pub SessionUser);

impl<S> axum::extract::FromRequestParts<S> for Auth
where
    S: Send + Sync,
    Key: axum::extract::FromRef<S>,
{
    type Rejection = axum::http::StatusCode;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let jar = SignedCookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| axum::http::StatusCode::UNAUTHORIZED)?;
        extract(&jar)
            .map(Auth)
            .ok_or(axum::http::StatusCode::UNAUTHORIZED)
    }
}
