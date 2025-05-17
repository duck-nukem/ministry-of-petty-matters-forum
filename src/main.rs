use axum::{
    routing::get,
    Router,
};

mod petty_matters;
mod persistence;
mod error;

#[tokio::main]
#[allow(clippy::expect_used)]
async fn main() {
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.expect("Field to start the server, maybe the port is already in use");
    axum::serve(listener, app).await.expect("Failed to start the server. This sucks!");
}