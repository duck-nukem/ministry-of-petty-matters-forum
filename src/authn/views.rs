use crate::authn::oauth::config::OAuthProvider;
use crate::authn::oauth::token::validate_token;
use crate::authn::session::User;
use crate::error::AnyError;
use crate::render_template;
use crate::view::HtmlResponse;
use crate::view::{notify_maintainers_on_error, show_error_page};
use askama::Template;
use axum::http::header::SET_COOKIE;
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Form, Router};
use serde::{Deserialize, Serialize};
use crate::config::APP_CONFIG;

#[derive(Template)]
#[template(path = "authn/login.html")]
pub struct LoginPage {
    root: String
}

async fn render_login_view() -> Result<HtmlResponse, StatusCode> {
    let template = render_template!(LoginPage {root: APP_CONFIG.get_root_url()});
    Ok(HtmlResponse::from_string(template))
}

#[derive(Debug, Deserialize, Serialize)]
struct OauthResponse {
    credential: String,
}

async fn oauth_callback(Form(body): Form<OauthResponse>) -> Response {
    let provider = OAuthProvider::Google;
    let claims = match validate_token(body.credential.as_str(), provider).await {
        Ok(v) => v,
        Err(e) => {
            return handle_authentication_failure(provider, &e.into());
        }
    };
    let Some(email) = claims.email else {
        return handle_authentication_failure(provider, &AnyError::from("e-mail was not present in the token"));
    };
    let user = User::new(email,  claims.exp);
    let session_cookie = match user.into_cookie() {
        Ok(cookie) => cookie,
        Err(e) => {
            return handle_authentication_failure(provider, &e);
        }
    };
    let mut response = Redirect::to("/").into_response();
    if let Ok(cookie_header) = HeaderValue::from_str(session_cookie.as_str()) {
        response.headers_mut().insert(SET_COOKIE, cookie_header);
    } else {
        return handle_authentication_failure(provider, &AnyError::from("Invalid cookie header"));
    }

    response
}

fn handle_authentication_failure(provider: OAuthProvider, e: &AnyError) -> Response {
    notify_maintainers_on_error(e);
    let mut response = Redirect::to("/auth?error=invalid_token").into_response();
    delete_cookies(provider.get_session_cookie_names(), &mut response);

    response
}

#[allow(clippy::needless_pass_by_value)]
fn delete_cookies(cookie_names: Vec<&str>, response: &mut Response) {
    let () = cookie_names.iter().for_each(|name| {
        if let Ok(header_value) = HeaderValue::from_str(
            format!("{name}=; Max-Age=0; Path=/; HttpOnly; SameSite=Lax").as_str(),
        ) {
            response.headers_mut().append(SET_COOKIE, header_value);
        }
    });
}

pub fn auth_router() -> Router {
    Router::new()
        .route("/", get(render_login_view))
        .route("/callback", post(oauth_callback))
}
