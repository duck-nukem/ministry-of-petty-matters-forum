use crate::authn::views::auth_router;
use crate::config::APP_CONFIG;
use crate::persistence::in_memory_repository::InMemoryRepository;
use crate::petty_matters::views::topics_router;
use axum::response::Redirect;
use axum::{routing::get, Router};
use petty_matters::service::TopicService;
use std::sync::Arc;
use sea_orm::{Database, DatabaseConnection};
use tokio::sync::mpsc::channel;
use tower_http::services::ServeDir;
use crate::petty_matters::repository::TopicRepository;
use crate::queue::in_memory_queue::WriteQueue;
use crate::queue::worker::start_write_worker;

mod authn;
mod config;
mod error;
mod persistence;
mod petty_matters;
mod templates;
mod time;
mod view;
mod queue;

static MAIN_ENTRY_POINT: &str = "/petty-matters";

#[tokio::main]
#[allow(clippy::expect_used)]
async fn main() {
    let db: DatabaseConnection = Database::connect(&APP_CONFIG.database_url).await.expect("Failed to connect to the database");
    let (tx, rx) = channel(100);
    let write_queue = Arc::new(WriteQueue::new(tx.clone()));

    let topic_repository = Arc::new(TopicRepository { db: db.clone() });
    let comment_repository = Arc::new(InMemoryRepository::new());
    tokio::spawn(start_write_worker(rx, topic_repository.clone(), comment_repository.clone()));
    let topic_service = Arc::new(TopicService::new(topic_repository, comment_repository, write_queue));

    let app = Router::new()
        .route("/", get(|| async { Redirect::to(MAIN_ENTRY_POINT) }))
        .nest("/auth", auth_router())
        .nest(MAIN_ENTRY_POINT, topics_router(topic_service))
        .nest_service("/assets", ServeDir::new("assets"));
    let address = APP_CONFIG.get_address();
    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .expect("Failed to bind listener on host & port, maybe the port is already in use?");
    axum::serve(listener, app)
        .await
        .expect("Failed to start the server (x_x')");
}
