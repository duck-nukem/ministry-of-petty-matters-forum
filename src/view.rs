use crate::error::AnyError;
use crate::time::Seconds;
use askama::Template;
use axum::http::StatusCode;
use axum::response::Html;
use crate::templates::Nonce;
use crate::views::templates::HtmlResponse;

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
