use crate::authn::oauth::config::OAuthProvider;
use crate::authn::oauth::token::validate_token;
use crate::authn::session::{User, Username};
use crate::config::APP_CONFIG;
use crate::error::AnyError;
use crate::error::notify_maintainers_on_error;
use crate::render_template;
use crate::templates::Nonce;
use crate::views::templates::HtmlResponse;
use askama::Template;
use axum::http::header::SET_COOKIE;
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Form, Router};
use serde::{Deserialize, Serialize};

#[derive(Template)]
#[template(path = "authn/login.html")]
pub struct LoginPage {
    root: String,
    user: User,
    nonce: Nonce,
}

async fn render_login_view(nonce: Nonce, user: User) -> Result<HtmlResponse, StatusCode> {
    let template = render_template!(LoginPage {
        root: APP_CONFIG.public_root_url.clone(),
        nonce,
        user
    });
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
        return handle_authentication_failure(
            provider,
            &AnyError::from("e-mail was not present in the token"),
        );
    };
    let user = User::new(Username(email), claims.exp);
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

async fn perform_logout() -> Result<Response, StatusCode> {
    let mut response = Redirect::to("/").into_response();
    delete_cookies(
        OAuthProvider::Google.get_session_cookie_names(),
        &mut response,
    );

    Ok(response)
}

pub fn auth_router() -> Router {
    Router::new()
        .route("/", get(render_login_view))
        .route("/logout", post(perform_logout))
        .route("/callback", post(oauth_callback))
}
