use axum::response::{Html, IntoResponse, Response};
use axum::http::{header, StatusCode};
use crate::time::Seconds;

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