use std::net::SocketAddr;
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::format;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use axum::{
    Router,
    routing::get,
    extract::{State, Query},
    Json,
    response::IntoResponse,
    http::StatusCode,
};

struct AppContext {
    categories: Arc<Mutex<Vec<String>>>,
}

#[derive(Deserialize)]
struct CategoryDataModel {
    name: Option<String>,
}

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

    // Common application context for all routes
    let context = Arc::new(AppContext {
        categories: Arc::new(Mutex::new(vec!["Food".to_string(), "Common".to_string()]))
    });

    // All the routes will be here
    // Will be split into modules later
    let app = Router::new()
    	.route("/", get(hello_world))
        .route("/hello", get(hello))
        .route("/categories", get(get_categories).post(add_category))
        .with_state(context); // Attach the application context to the router

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

async fn get_categories(State(ctx): State<Arc<AppContext>>) -> impl IntoResponse {
	let categories = ctx.categories.lock().unwrap();
    let categories_list: Vec<String> = categories.clone();
    Json(categories_list)
}

async fn add_category(
    State(ctx): State<Arc<AppContext>>,
    category: Option<Json<CategoryDataModel>>) -> impl IntoResponse {
    if let Some(Json(category)) = category {
        if let Some(name) = category.name {
            if name.trim().is_empty() {
                return (StatusCode::BAD_REQUEST, Json("Category name is required".to_string()))
            }

            let mut categories = ctx.categories.lock().unwrap();
            if categories.contains(&name) {
                return (StatusCode::CONFLICT, Json(format!("Category already exists: {name}")));
            }

            categories.push(name.clone());
            return (StatusCode::CREATED, Json(format!("Category added successfully: {name}")));
        }
    }

    (StatusCode::BAD_REQUEST, Json("Category name is required".to_string()))
}