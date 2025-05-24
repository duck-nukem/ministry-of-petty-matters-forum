use crate::authn::oauth::config::{OAuthConfig, OAuthProvider};
use crate::authn::oauth::errors::TokenValidationError;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashSet;

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

async fn fetch_jwks(certs_url: &str) -> Jwks {
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

pub async fn validate_token(
    token: &str,
    provider: OAuthProvider,
) -> Result<Claims, TokenValidationError> {
    let oauth_config = OAuthConfig::for_provider(provider);
    let header = decode_header(token).map_err(|_| TokenValidationError::InvalidHeader)?;
    let kid = header.kid.ok_or(TokenValidationError::MissingKid)?;
    let jwks = fetch_jwks(oauth_config.certs_url).await;
    let jwk = find_key(&jwks, &kid).ok_or(TokenValidationError::KeyNotFound)?;
    let key = decoding_key(jwk).map_err(|_| TokenValidationError::InvalidKey)?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[oauth_config.client_id]);
    validation.iss = Some(HashSet::from([oauth_config.issuer.to_string()]));

    let token_data = decode::<Claims>(token, &key, &validation)
        .map_err(|_| TokenValidationError::TokenDecode)?;
    Ok(token_data.claims)
}
