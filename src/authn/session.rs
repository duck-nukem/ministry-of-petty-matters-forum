use crate::config::APP_CONFIG;
use crate::error::Result;
use crate::time::{Days, Seconds, TimeUnit};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::sync::LazyLock;

pub static SESSION_COOKIE_NAME: &str = "session";

#[allow(clippy::expect_used)]
static ENCODING_KEY: LazyLock<EncodingKey> = LazyLock::new(|| {
    EncodingKey::from_base64_secret(&APP_CONFIG.secret).expect("Failed to create encoding key")
});
#[allow(clippy::expect_used)]
static DECODING_KEY: LazyLock<DecodingKey> = LazyLock::new(|| {
    DecodingKey::from_base64_secret(&APP_CONFIG.secret).expect("Failed to create decoding key")
});
static TOKEN_LIFETIME: LazyLock<Seconds> = LazyLock::new(|| Days(14).to_seconds());

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Username(pub String);

impl Default for Username {
    fn default() -> Self {
        Self(String::from("anonymous@localhost"))
    }
}

impl Display for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub email: Username, // name mustn't change; must overlap with a "Claim"
    pub exp: usize,      // name mustn't change; must overlap with a "Claim"
    pub is_anonymous: bool,
}

impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "User(email: {}, is_anonymous: {})",
            self.email, self.is_anonymous
        )
    }
}

impl User {
    pub fn into_cookie(self) -> Result<String> {
        let token = encode_user_data(&self)?;
        let lifetime = TOKEN_LIFETIME.to_string();
        Ok(format!(
            "{SESSION_COOKIE_NAME}={token}; \
            Max-Age={lifetime}; Path=/; \
            HttpOnly; SameSite=Lax"
        ))
    }

    pub const fn new(email: Username, expires_at: usize) -> Self {
        Self {
            email,
            exp: expires_at,
            is_anonymous: false,
        }
    }

    pub fn anonymous() -> Self {
        Self {
            email: Username::default(),
            exp: 0,
            is_anonymous: true,
        }
    }
}

pub fn encode_user_data(user: &User) -> jsonwebtoken::errors::Result<String> {
    encode(&Header::default(), user, &ENCODING_KEY)
}

pub fn decode_user_data(token: &str) -> jsonwebtoken::errors::Result<User> {
    decode::<User>(token, &DECODING_KEY, &Validation::new(Algorithm::HS256))
        .map(|token| token.claims)
}
