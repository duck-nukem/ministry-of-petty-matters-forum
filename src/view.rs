use crate::error::AnyError;
use crate::time::Seconds;
use askama::Template;
use axum::http::{header, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use crate::templates::Nonce;

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

#[macro_export]
macro_rules! render_template {
    ($template:expr) => {
        match $template.render() {
            Ok(t) => t,
            Err(e) => return show_error_page(Box::new(e)),
        }
    };
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
    notify_maintainers_on_error(&error.into());
    let response = render_template!(InternalServerErrorPage { nonce: Nonce::new() });

    Ok(HtmlResponse {
        response: Html(response),
        status_code: Some(StatusCode::INTERNAL_SERVER_ERROR),
        max_age: None,
    })
}

pub fn notify_maintainers_on_error(error: &AnyError) {
    eprintln!("Error occurred: {error}");
}

pub fn show_not_found_page() -> Result<HtmlResponse, StatusCode> {
    let response = render_template!(NotFoundErrorPage { nonce: Nonce::new() });

    Ok(HtmlResponse {
        response: Html(response),
        status_code: Some(StatusCode::NOT_FOUND),
        max_age: Some(Seconds(60)),
    })
}
