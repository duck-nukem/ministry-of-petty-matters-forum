use crate::petty_matters::views::topics_router;
use axum::response::Html;
use axum::{
    routing::get,
    Router,
};
use std::sync::Arc;
use petty_matters::service::TopicService;
use crate::persistence::in_memory_repository::InMemoryRepository;

mod petty_matters;
mod persistence;
mod error;


#[tokio::main]
#[allow(clippy::expect_used)]
async fn main() {
    let topic_repository = Arc::new(InMemoryRepository::new());
    let topic_service = Arc::new(TopicService::new(topic_repository));

    let app = Router::new()
        .route("/", get(|| async { Html("Hello, world!") }))
        .nest("/petty-matters", topics_router(topic_service));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.expect("Field to start the server, maybe the port is already in use");
    axum::serve(listener, app).await.expect("Failed to start the server. This sucks!");
}