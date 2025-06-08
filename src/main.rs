use crate::authn::views::auth_router;
use crate::config::APP_CONFIG;
use crate::error::AnyError;
use crate::petty_matters::views::petty_matters_router;
use axum::response::Redirect;
use axum::{Router, routing::get};
use petty_matters::service;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use std::time::Duration;
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
async fn main() -> Result<(), AnyError> {
    println!("Starting up");

    let database_connection = connect_to_database(&APP_CONFIG.database_url).await;
    let petty_matters_service = service::petty_matters_service_factory(database_connection)?;

    println!("Configuring routes and middlewares");
    let app = Router::new()
        .route("/", get(|| async { Redirect::to(MAIN_ENTRY_POINT) }))
        .nest("/auth", auth_router())
        .nest(
            MAIN_ENTRY_POINT,
            petty_matters_router(petty_matters_service),
        )
        .nest_service("/assets", ServeDir::new("assets"));

    run_server(app, APP_CONFIG.get_address()).await?;

    Ok(())
}

async fn connect_to_database(database_url: &String) -> Result<DatabaseConnection, DbErr> {
    println!("Attempting to connect to the database");
    let mut connection_options = ConnectOptions::new(database_url);
    connection_options
        .min_connections(5)
        .max_connections(20)
        .connect_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(30))
        .sqlx_logging(false);

    Database::connect(connection_options).await
}

async fn run_server(app: Router, address: String) -> Result<(), AnyError> {
    println!("Starting listener on {address}");
    let listener = tokio::net::TcpListener::bind(&address).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
