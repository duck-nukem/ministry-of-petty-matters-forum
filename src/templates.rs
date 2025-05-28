use std::fmt::{Display, Formatter};
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use uuid::Uuid;

pub mod filters {
    #[allow(clippy::unnecessary_wraps)]
    pub fn markdown<T: std::fmt::Display>(
        s: T,
        _: &dyn askama::Values,
    ) -> askama::Result<String> {
        let text = &s.to_string();
        let parser = pulldown_cmark::Parser::new_ext(text, pulldown_cmark::Options::all());

        let mut buf = String::new();
        pulldown_cmark::html::push_html(&mut buf, parser);
        let result = buf.replace("<img", "<img loading=\"lazy\"");

        Ok(result)
    }
}

pub struct Nonce(String);

impl Display for Nonce {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Nonce {
    pub fn new() -> Self {
        Self(Uuid::new_v4().simple().to_string())
    }
}

impl<S> FromRequestParts<S> for Nonce
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(_parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self::new())
    }
}
