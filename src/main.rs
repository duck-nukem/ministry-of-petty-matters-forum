use crate::authn::views::auth_router;
use crate::config::APP_CONFIG;
use crate::persistence::in_memory_repository::InMemoryRepository;
use crate::petty_matters::views::topics_router;
use askama::Template;
use axum::response::Redirect;
use axum::{routing::get, Router};
use petty_matters::service::TopicService;
use std::sync::Arc;
use tower_http::services::ServeDir;

mod authn;
mod config;
mod error;
mod persistence;
mod petty_matters;
mod time;
mod view;

#[derive(Template)]
#[template(path = "errors/404.html")]
pub struct NotFoundPage {}

static MAIN_ENTRY_POINT: &str = "/petty-matters";

#[tokio::main]
#[allow(clippy::expect_used)]
async fn main() {
    let topic_repository = Arc::new(InMemoryRepository::new());
    let comment_repository = Arc::new(InMemoryRepository::new());
    let topic_service = Arc::new(TopicService::new(topic_repository, comment_repository));

    let app = Router::new()
        .route("/", get(|| async { Redirect::to(MAIN_ENTRY_POINT) }))
        .nest("/auth", auth_router())
        .nest(MAIN_ENTRY_POINT, topics_router(topic_service))
        .nest_service("/assets", ServeDir::new("assets"));
    let address = APP_CONFIG.get_address();
    let listener = tokio::net::TcpListener::bind(&address)
    .await
    .expect(format!("Failed to listen on {address}, maybe the port is already in use?").as_str());
    axum::serve(listener, app)
        .await
        .expect("Failed to start the server (x_x')");
}
