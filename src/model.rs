use axum::http::StatusCode;
use axum::Json;
use sqlx::FromRow;
use sqlx::postgres::PgRow;
use crate::AppContext;

pub trait ModelController {
    const TABLE: &'static str;
}

pub async fn get<MC, E>(ctx: &AppContext, id: i64) -> Result<Option<E>, (StatusCode, Json<String>)>
where
    MC: ModelController,
    E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
{
    let sql = format!("SELECT * FROM {} WHERE id = $1", MC::TABLE);
    let entity = sqlx::query_as::<_, E>(&sql)
        .bind(id)
        .fetch_optional(&ctx.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())))?;

    Ok(entity)
}

pub async fn list<MC, E>(ctx: &AppContext) -> Result<Vec<E>, (StatusCode, Json<String>)>
where
    MC: ModelController,
    E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
{
    let sql = format!("SELECT * FROM {}", MC::TABLE);
    let entities = sqlx::query_as::<_, E>(&sql)
        .fetch_all(&ctx.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())))?;

    Ok(entities)
}

pub async fn delete<MC>(ctx: &AppContext, id: i64) -> Result<(), (StatusCode, Json<String>)>
where
    MC: ModelController,
{
    let sql = format!("DELETE FROM {} WHERE id = $1", MC::TABLE);
    sqlx::query(&sql)
        .bind(id)
        .execute(&ctx.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())))?;
    
    Ok(())
}

