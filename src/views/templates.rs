use askama::Template;
use axum::response::{Html, IntoResponse, Response};
use axum::http::{header, StatusCode};
use crate::error::AnyError;
use crate::templates::Nonce;
use crate::time::Seconds;
use crate::{error, render_template};

pub struct HtmlResponse {
    pub response: Html<String>,
    pub status_code: Option<StatusCode>,
    pub max_age: Option<Seconds>,
}

impl HtmlResponse {
    pub const fn from_string(response: String) -> Self {
        Self {
            response: Html(response),
            status_code: Some(StatusCode::OK),
            max_age: None,
        }
    }

    pub const fn cached(response: String, cache_for: Seconds) -> Self {
        Self {
            response: Html(response),
            status_code: Some(StatusCode::OK),
            max_age: Some(cache_for),
        }
    }
}

impl IntoResponse for HtmlResponse {
    fn into_response(self) -> Response {
        let response_with_status = match self.status_code {
            Some(status_code) => (status_code, self.response),
            None => (StatusCode::OK, self.response),
        };
        let mut res = response_with_status.into_response();
        if let Some(seconds) = self.max_age {
            res.headers_mut().insert(
                header::CACHE_CONTROL,
                header::HeaderValue::from_str(&format!("max-age={}", seconds.0))
                    .unwrap_or(header::HeaderValue::from_static("max-age=60")),
            );
        }
        res
    }
}

#[derive(Template)]
#[template(path = "errors/5xx.html")]
pub struct InternalServerErrorPage {
    nonce: Nonce,
}

#[derive(Template)]
#[template(path = "errors/404.html")]
pub struct NotFoundErrorPage {
    nonce: Nonce,
}

pub fn show_error_page<E>(error: E) -> Result<HtmlResponse, StatusCode>
where
    E: Into<AnyError>,
{
    error::notify_maintainers_on_error(&error.into());
    let response = render_template!(InternalServerErrorPage { nonce: Nonce::new() });

    Ok(HtmlResponse {
        response: Html(response),
        status_code: Some(StatusCode::INTERNAL_SERVER_ERROR),
        max_age: None,
    })
}

pub fn show_not_found_page() -> Result<HtmlResponse, StatusCode> {
    let response = render_template!(NotFoundErrorPage { nonce: Nonce::new() });

    Ok(HtmlResponse {
        response: Html(response),
        status_code: Some(StatusCode::NOT_FOUND),
        max_age: Some(Seconds(60)),
    })
}