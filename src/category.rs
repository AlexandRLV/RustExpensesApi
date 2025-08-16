use std::sync::Arc;
use axum::http::StatusCode;
use axum::{Json, Router};
use axum::extract::State;
use serde::Serialize;
use sqlx::FromRow;
use crate::{model, AppContext};
use crate::model::ModelController;

#[derive(FromRow, Serialize, Debug)]
pub struct Category {
    id: i32,
    user_id: i32,
    name: String,
}

pub struct CategoryModelController { }

pub fn categories_router() -> Router {
    Router::new()
}

async fn get_categories(State(ctx): State<Arc<AppContext>>) -> Result<Json<Vec<String>>, (StatusCode, Json<String>)> {
    let categories = ctx.category_mc.list(&ctx).await;
}

impl ModelController for CategoryModelController {
    const TABLE: &'static str = "user_category";
}

impl CategoryModelController {
    pub async fn list(ctx: &AppContext) -> Result<Vec<Category>, (StatusCode, Json<String>)> {
        model::list::<Self, _>(ctx).await
    }

    pub async fn create(ctx: &AppContext, category: &Category) -> Result<Category, (StatusCode, Json<String>)> {
        let sql = format!("INSERT INTO {} (user_id, name) VALUES ($1, $2) RETURNING *", Self::TABLE);
        let new_category = sqlx::query_as::<_, Category>(&sql)
            .bind(&category.user_id).bind(&category.name)
            .fetch_one(&ctx.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())))?;

        Ok(new_category)
    }

    pub async fn update(ctx: &AppContext, category: &Category) -> Result<Category, (StatusCode, Json<String>)> {
        let sql = format!("UPDATE {} SET user_id = $1, name = $2 WHERE (id = $3) RETURNING *", Self::TABLE);
        let new_category = sqlx::query_as::<_, Category>(&sql)
            .bind(&category.user_id).bind(&category.name).bind(&category.id)
            .fetch_one(&ctx.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())))?;

        Ok(new_category)
    }
}