use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::LazyLock;
use crate::error::Result;

pub static SESSION_COOKIE_NAME: &str = "session";
static DEVELOPMENT_ENCRYPTION_KEY: &str = "czNjcjN0LXMzY3IzdC1zM2NyM3QtczNjcjN0LXMzY3IzdA==";

#[allow(clippy::expect_used)]
static ENCODING_KEY: LazyLock<EncodingKey> = LazyLock::new(|| {
    let base64_key = env::var("JWT_SECRET").unwrap_or_else(|_| DEVELOPMENT_ENCRYPTION_KEY.to_string());
    EncodingKey::from_base64_secret(&base64_key).expect("Failed to create encoding key")
});
#[allow(clippy::expect_used)]
static DECODING_KEY: LazyLock<DecodingKey> = LazyLock::new(|| {
    let base64_key = env::var("JWT_SECRET").unwrap_or_else(|_| DEVELOPMENT_ENCRYPTION_KEY.to_string());
    DecodingKey::from_base64_secret(&base64_key).expect("Failed to create decoding key")
});

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub email: String,
    pub expires_at: usize,
}

impl User {
    pub fn into_cookie(self) -> Result<String> {
        let user_data = encode_user_data(&self)?;
        Ok(format!("{SESSION_COOKIE_NAME}={user_data}; Max-Age=3600; HttpOnly; Secure; SameSite=Strict"))
    }
}

pub fn encode_user_data(user: &User) -> jsonwebtoken::errors::Result<String> {
    encode(&Header::default(), user, &ENCODING_KEY)
}

pub fn decode_user_data(token: &str) -> jsonwebtoken::errors::Result<User> {
    decode::<User>(token, &DECODING_KEY, &Validation::new(Algorithm::HS256))
        .map(|token| token.claims)
}
