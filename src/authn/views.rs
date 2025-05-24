use crate::authn::oauth::{validate_token, OAuthProvider};
use crate::render_template;
use crate::view::show_error_page;
use crate::view::HtmlResponse;
use askama::Template;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

#[derive(Template)]
#[template(path = "authn/login.html")]
pub struct LoginPage {}

async fn render_login_view() -> Result<HtmlResponse, StatusCode> {
    let template = render_template!(LoginPage {});
    Ok(HtmlResponse::from_string(template))
}

#[derive(Debug, Deserialize, Serialize)]
struct OauthResponse {
    credential: String,
}

async fn oauth_callback(Json(body): Json<OauthResponse>) -> Result<HtmlResponse, StatusCode> {
    let claims = match validate_token(body.credential.as_str(), OAuthProvider::Google).await {
        Ok(v) => v,
        Err(e) => {
            // TODO: Delete cookies
            return show_error_page(e);
        }
    };
    println!("{claims:?}");
    // TODO: store valid credentials in a session store
    Ok(HtmlResponse::from_string("OAuth callback not implemented yet".to_string()))
}

pub fn auth_router() -> Router {
    Router::new()
        .route("/", get(render_login_view))
        .route("/callback", post(oauth_callback))
}
