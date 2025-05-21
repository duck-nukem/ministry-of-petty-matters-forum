use crate::persistence::in_memory_repository::InMemoryRepository;
use crate::petty_matters::views::topics_router;
use askama::Template;
use axum::response::Redirect;
use axum::{routing::get, Router};
use petty_matters::service::TopicService;
use std::sync::Arc;
use tower_http::services::ServeDir;

mod error;
mod persistence;
mod petty_matters;
mod time;
mod view;

#[derive(Template)]
#[template(path = "errors/404.html")]
pub struct NotFoundPage {}

#[tokio::main]
#[allow(clippy::expect_used)]
async fn main() {
    let topic_repository = Arc::new(InMemoryRepository::new());
    let comment_repository = Arc::new(InMemoryRepository::new());
    let topic_service = Arc::new(TopicService::new(topic_repository, comment_repository));

    let app = Router::new()
        .route("/", get(|| async { Redirect::to("/petty-matters") }))
        .nest("/petty-matters", topics_router(topic_service))
        .nest_service("/assets", ServeDir::new("assets"));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Field to start the server, maybe the port is already in use");
    axum::serve(listener, app)
        .await
        .expect("Failed to start the server. This sucks!");
}
