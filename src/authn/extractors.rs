use crate::authn::session::{SESSION_COOKIE_NAME, User, decode_user_data};
use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::http::header::COOKIE;
use axum::http::request::Parts;

static COOKIE_SEPARATOR: &str = ";";

impl<S> FromRequestParts<S> for User
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts.headers.get(COOKIE).map_or_else(
            || Ok(Self::anonymous()),
            |cookies| {
                let mut session_cookie = cookies
                    .to_str()
                    .unwrap_or("")
                    .split(COOKIE_SEPARATOR)
                    .filter_map(|cookie| {
                        if cookie.trim().starts_with(SESSION_COOKIE_NAME) {
                            Some(cookie.trim().split('=').nth(1).unwrap_or(""))
                        } else {
                            None
                        }
                    });

                session_cookie.next().map_or_else(
                    || Ok(Self::anonymous()),
                    |c| Ok(decode_user_data(c).unwrap_or_else(|_| Self::anonymous())),
                )
            },
        )
    }
}
