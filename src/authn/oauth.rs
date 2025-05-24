use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::fmt::Display;

pub static GOOGLE_OAUTH_CLIENT_ID: &str =
    "344287748721-uvjlsv8iul7a2f40eviipeknj4tpaea3.apps.googleusercontent.com";
static GOOGLE_OAUTH_CERTS_URL: &str = "https://www.googleapis.com/oauth2/v3/certs";
static GOOGLE_OAUTH_ISSUER: &str = "https://accounts.google.com";

#[derive(Debug)]
pub enum TokenValidationError {
    InvalidHeader,
    MissingKid,
    KeyNotFound,
    InvalidKey,
    TokenDecode,
}

impl Display for TokenValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OAuth Token Validation failed ~> {self:?}")
    }
}

impl Error for TokenValidationError {}

#[derive(Debug, Deserialize)]
struct Jwk {
    // https://datatracker.ietf.org/doc/html/rfc7517
    kid: String,
    n: String,
    e: String,
}

#[derive(Debug, Deserialize)]
struct Jwks {
    keys: Vec<Jwk>,
}

async fn fetch_google_jwks(certs_url: &str) -> Jwks {
    let Ok(response) = Client::new().get(certs_url).send().await else {
        return Jwks { keys: vec![] };
    };

    response
        .json::<Jwks>()
        .await
        .unwrap_or_else(|_| Jwks { keys: vec![] })
}

fn find_key<'a>(jwks: &'a Jwks, kid: &str) -> Option<&'a Jwk> {
    jwks.keys.iter().find(|k| k.kid == kid)
}

fn decoding_key(jwk: &Jwk) -> jsonwebtoken::errors::Result<DecodingKey> {
    DecodingKey::from_rsa_components(&jwk.n, &jwk.e)
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Claims {
    sub: String,
    aud: String,
    exp: usize,
    iss: String,
    email: Option<String>,
}

pub enum OAuthProvider {
    Google,
}

pub async fn validate_token(
    token: &str,
    provider: OAuthProvider,
) -> Result<Claims, TokenValidationError> {
    let (issuer, client_id, certs_url) = match provider {
        OAuthProvider::Google => (
            GOOGLE_OAUTH_ISSUER,
            GOOGLE_OAUTH_CLIENT_ID,
            GOOGLE_OAUTH_CERTS_URL,
        ),
    };
    let header = decode_header(token).map_err(|_| TokenValidationError::InvalidHeader)?;
    let kid = header.kid.ok_or(TokenValidationError::MissingKid)?;
    let jwks = fetch_google_jwks(certs_url).await;
    let jwk = find_key(&jwks, &kid).ok_or(TokenValidationError::KeyNotFound)?;
    let key = decoding_key(jwk).map_err(|_| TokenValidationError::InvalidKey)?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[client_id]);
    validation.iss = Some(HashSet::from([issuer.to_string()]));

    let token_data = decode::<Claims>(token, &key, &validation)
        .map_err(|_| TokenValidationError::TokenDecode)?;
    Ok(token_data.claims)
}
