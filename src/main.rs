use crate::authn::views::auth_router;
use crate::config::APP_CONFIG;
use crate::error::AnyError;
use crate::feature_flags::FEATURE_FLAGS;
use crate::persistence::in_memory_repository::InMemoryRepository;
use crate::persistence::rdbms::RdbmsRepository;
use crate::persistence::repository::Repository;
use crate::petty_matters::comment::{Comment, CommentId};
use crate::petty_matters::comment_repository::Entity as CommentDbModel;
use crate::petty_matters::topic::{Topic, TopicId};
use crate::petty_matters::topic_repository::Entity as TopicDbModel;
use crate::petty_matters::views::topics_router;
use crate::queue::in_memory_queue::WriteQueue;
use crate::queue::worker::start_write_worker;
use axum::response::Redirect;
use axum::{Router, routing::get};
use petty_matters::service::PettyMattersService;
use sea_orm::{ConnectOptions, Database};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::channel;
use tower_http::services::ServeDir;

mod authn;
mod config;
mod error;
mod feature_flags;
mod persistence;
mod petty_matters;
mod queue;
mod templates;
mod time;
mod views;

static MAIN_ENTRY_POINT: &str = "/petty-matters";

#[tokio::main]
#[allow(clippy::expect_used, clippy::panic)]
async fn main() -> Result<(), AnyError> {
    println!("Starting up");
    let (tx, rx) = channel(100);
    let write_queue = Arc::new(WriteQueue::new(tx.clone()));

    let mut connection_options = ConnectOptions::new(&APP_CONFIG.database_url);
    connection_options
        .min_connections(5)
        .max_connections(20)
        .connect_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(30))
        .sqlx_logging(false);

    println!("Attempting to connect to the database");
    let topic_repository: Arc<dyn Repository<TopicId, Topic> + Send + Sync>;
    let comment_repository: Arc<dyn Repository<CommentId, Comment> + Send + Sync>;
    match Database::connect(connection_options).await {
        Ok(db) => {
            println!("Connection established");
            topic_repository = Arc::new(RdbmsRepository::<TopicDbModel>::new(db.clone()));
            comment_repository = Arc::new(RdbmsRepository::<CommentDbModel>::new(db));
        }
        Err(e) => {
            if FEATURE_FLAGS.is_ephemeral_db_allowed {
                eprintln!(
                    "Database connection failed: {e},
                    using in-memory repositories as fallback.
                    If you'd like to disallow the fallback behavior,
                    set the EPHEMERAL_DB_ALLOWED environment variable to false."
                );
                topic_repository = Arc::new(InMemoryRepository::<TopicId, Topic>::new());
                comment_repository = Arc::new(InMemoryRepository::<CommentId, Comment>::new());
            } else {
                eprintln!(
                    "Database connection failed: {e}
                    and ephemeral DB is not allowed, exiting.
                    To allow the application to run with an in-memory database,
                    set the EPHEMERAL_DB_ALLOWED environment variable to true."
                );
                return Err(e.into());
            }
        }
    }

    println!("Wiring services");
    tokio::spawn(start_write_worker(
        rx,
        topic_repository.clone(),
        comment_repository.clone(),
    ));
    let topic_service = Arc::new(PettyMattersService::new(
        topic_repository,
        comment_repository,
        write_queue,
    ));
    println!("Service configuration done");

    println!("Configuring routes and middlewares");
    let app = Router::new()
        .route("/", get(|| async { Redirect::to(MAIN_ENTRY_POINT) }))
        .nest("/auth", auth_router())
        .nest(MAIN_ENTRY_POINT, topics_router(topic_service))
        .nest_service("/assets", ServeDir::new("assets"));
    println!("Done; attempting to start the server");
    let address = APP_CONFIG.get_address();
    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .expect("Failed to bind listener on host & port, maybe the port is already in use?");
    axum::serve(listener, app)
        .await
        .expect("Failed to start the server (x_x')");
    Ok(())
}
