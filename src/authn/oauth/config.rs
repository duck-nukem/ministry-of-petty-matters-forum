use crate::authn::oauth::providers::google::{GOOGLE_OAUTH_CERTS_URL, GOOGLE_OAUTH_CLIENT_ID, GOOGLE_OAUTH_ISSUER};

pub enum OAuthProvider {
    Google,
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