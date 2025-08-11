use std::net::SocketAddr;
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::format;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use sqlx::postgres::{PgPool, PgPoolOptions};
use axum::{
    Router,
    routing::get,
    extract::{State, Query},
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use sqlx::FromRow;

struct AppContext {
    categories: Arc<Mutex<Vec<String>>>,
    db: PgPool,
}

#[derive(FromRow, Serialize)]
struct Category {
    id: i32,
    name: String,
}

#[derive(Deserialize)]
struct CategoryDataModel {
    id: Option<i32>,
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

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://expenses-user:password@localhost:5432/expenses-db")
        .await
        .expect("Failed to connect to the database");

    // Common application context for all routes
    let context = Arc::new(AppContext {
        categories: Arc::new(Mutex::new(vec!["Food".to_string(), "Common".to_string()])),
        db: pool,
    });

    // All the routes will be here
    // Will be split into modules later
    let app = Router::new()
    	.route("/", get(hello_world))
        .route("/hello", get(hello))
        .route("/categories", get(get_categories).post(add_category).delete(delete_category))
        .route("/categories-db", get(get_categories_db).post(add_category_db).delete(delete_category_db))
        .route("/category-db", get(get_category_by_id_db))
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
    category: Option<Json<CategoryDataModel>>) -> impl IntoResponse
{
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

async fn delete_category(
    State(ctx): State<Arc<AppContext>>,
    category: Option<Json<CategoryDataModel>>) -> impl IntoResponse
{
    if let Some(Json(category)) = category {
        if let Some(name) = category.name {
            if name.trim().is_empty() {
                return (StatusCode::BAD_REQUEST, Json("Category name is required".to_string()))
            }

            let mut categories = ctx.categories.lock().unwrap();
            if let Some(index) = categories.iter().position(|value| **value == name) {
                categories.swap_remove(index);
                return (StatusCode::OK, Json(format!("Category deleted successfully: {name}")));
            }

            return (StatusCode::CREATED, Json(format!("Category doesn't exist: {name}")));
        }
    }

    (StatusCode::BAD_REQUEST, Json("Category name is required".to_string()))
}

async fn get_categories_db(State(ctx): State<Arc<AppContext>>) -> Result<Json<Vec<String>>, (StatusCode, Json<String>)> {
    let categories = sqlx::query_as::<_, Category>("SELECT * FROM common_categories")
        .fetch_all(&ctx.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())))?;

    let categories_list = categories.into_iter().map(|c| c.name).collect();
    Ok(Json(categories_list))
}

async fn get_category_by_id_db(State(ctx): State<Arc<AppContext>>, Query(dataModel): Query<CategoryDataModel>) -> Result<Json<Category>, (StatusCode, Json<String>)> {
    if let Some(id) = dataModel.id {
        tracing::info!("Requesting category by id {id}");
        let category = sqlx::query_as::<_, Category>("SELECT * FROM common_categories WHERE id = $1")
            .bind(id)
            .fetch_optional(&ctx.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())))?;

        match category {
            Some(category) => {
                Ok(Json(category))
            },
            None => Err((StatusCode::NOT_FOUND, Json("Category not found".to_string()))),
        }
    }
    else {
        Err((StatusCode::BAD_REQUEST, Json("Category ID is required".to_string())))
    }
}

async fn add_category_db(State(ctx): State<Arc<AppContext>>, category: Option<Json<CategoryDataModel>>) -> Result<Json<Category>, (StatusCode, Json<String>)> {
    if let Some(Json(category)) = category {
        if let Some(name) = category.name {
            if name.trim().is_empty() {
                return Err((StatusCode::BAD_REQUEST, Json("Category name is required".to_string())));
            }

            let existing_category = sqlx::query_as::<_, Category>("SELECT * FROM common_categories WHERE name = $1")
                .bind(&name)
                .fetch_optional(&ctx.db)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())))?;

            if existing_category.is_some() {
                return Err((StatusCode::CONFLICT, Json(format!("Category already exists: {name}"))));
            }

            let new_category = sqlx::query_as::<_, Category>("INSERT INTO common_categories (name) VALUES ($1) RETURNING *")
                .bind(name)
                .fetch_one(&ctx.db)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())))?;

            Ok(Json(new_category))
        } else {
            Err((StatusCode::BAD_REQUEST, Json("Category name is required".to_string())))
        }
    } else {
        Err((StatusCode::BAD_REQUEST, Json("Category data is required".to_string())))
    }
}

async fn delete_category_db(State(ctx): State<Arc<AppContext>>, category: Option<Json<CategoryDataModel>>) -> Result<Json<Category>, (StatusCode, Json<String>)> {
    if let Some(Json(category)) = category {
        if let Some(id) = category.id {
            let existing_category = sqlx::query_as::<_, Category>("SELECT * FROM common_categories WHERE id = $1")
                .bind(&id)
                .fetch_optional(&ctx.db)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())))?;

            if let Some(existing_category) = existing_category {
                sqlx::query("DELETE FROM common_categories WHERE id = $1")
                    .bind(existing_category.id)
                    .execute(&ctx.db)
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())))?;

                Ok(Json(existing_category))
            } else {
                Err((StatusCode::NOT_FOUND, Json(format!("Category not found: {id}"))))
            }
        } else if let Some(name) = category.name {
            let existing_category = sqlx::query_as::<_, Category>("SELECT * FROM common_categories WHERE name = $1")
                .bind(&name)
                .fetch_optional(&ctx.db)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())))?;

            if let Some(existing_category) = existing_category {
                sqlx::query("DELETE FROM common_categories WHERE id = $1")
                    .bind(existing_category.id)
                    .execute(&ctx.db)
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())))?;

                Ok(Json(existing_category))
            } else {
                Err((StatusCode::NOT_FOUND, Json(format!("Category not found: {name}"))))
            }
        } else {
            Err((StatusCode::BAD_REQUEST, Json("Category id or name is required".to_string())))
        }
    } else {
        Err((StatusCode::BAD_REQUEST, Json("Category id or name is required".to_string())))
    }
}