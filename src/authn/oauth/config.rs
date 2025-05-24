use crate::authn::oauth::providers::google::{GOOGLE_OAUTH_CERTS_URL, GOOGLE_OAUTH_CLIENT_ID, GOOGLE_OAUTH_ISSUER};
use crate::authn::session::SESSION_COOKIE_NAME;

#[derive(Copy, Clone)]
pub enum OAuthProvider {
    Google,
}

impl OAuthProvider {
    pub fn get_session_cookie_names(&self) -> Vec<&str> {
        match self {
            Self::Google => vec![SESSION_COOKIE_NAME, "g_state", "g_csrf_token"],
        }
    }
}

pub struct OAuthConfig<'a> {
    pub(crate) client_id: &'a str,
    pub(crate) issuer: &'a str,
    pub(crate) certs_url: &'a str,
}

impl OAuthConfig<'_> {
    pub fn for_provider(provider: OAuthProvider) -> Self {
        match provider {
            OAuthProvider::Google => OAuthConfig {
                client_id: GOOGLE_OAUTH_CLIENT_ID,
                issuer: GOOGLE_OAUTH_ISSUER,
                certs_url: GOOGLE_OAUTH_CERTS_URL,
            },
        }
    }
}