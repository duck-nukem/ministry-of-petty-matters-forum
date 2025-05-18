use crate::time::Seconds;
use axum::http::header;
use axum::response::IntoResponse;
use std::fmt::Display;

pub fn cache_response(
    original_response: impl IntoResponse,
    max_age: Option<Seconds>,
) -> impl IntoResponse {
    let mut response = original_response.into_response();
    let max_age_header_value = format!("max-age={}", max_age.unwrap_or(Seconds(60)).0);
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_str(&max_age_header_value)
            .unwrap_or(header::HeaderValue::from_static("max-age=60")),
    );
    response
}
