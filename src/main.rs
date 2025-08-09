use std::net::SocketAddr;
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::format;
use serde::{Deserialize, Serialize};
use axum::{
    Router,
    routing::get,
    extract::{State, Query},
    Json,
    response::IntoResponse,
    http::StatusCode,
};

#[derive(Deserialize)]
struct HelloQuery {
    name: Option<String>,
}

#[derive(Serialize)]
struct HelloResponse {
    message: String,
}

#[tokio::main]
async fn main() {
    fmt().init(); // for logging

    // All the routes will be here
    // Will be split into modules later
    let app = Router::new()
    	.route("/", get(hello_world))
        .route("/hello", get(hello));

    // Set listen address
    let addr: SocketAddr = ([0,0,0,0], 3000).into();

    // Logging
    tracing::info!("Starting server at http://{}", addr);

    // Start the server
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app) // bind to the address
    	.await
    	.expect("Failed to start server");
}

async fn hello_world() -> &'static str {
    "Hello, World! This is the Axum server running."
}

async fn hello(Query(query): Query<HelloQuery>) -> impl IntoResponse {
    let name = query.name.unwrap_or_else(|| "World".to_string());
    let response = HelloResponse {
        message: format!("Hello, {}!", name),
    };
    Json(response)
}