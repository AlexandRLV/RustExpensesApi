use std::net::SocketAddr;
use axum::{Router, routing::get, };
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::format;

#[tokio::main]
async fn main() {
    fmt().init(); // for logging

    // All the routes will be here
    // Will be split into modules later
    let app = Router::new()
    	.route("/", get(hello_world));

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