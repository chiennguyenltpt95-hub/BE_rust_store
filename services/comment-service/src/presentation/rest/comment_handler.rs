use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::response::ApiResponse;
use super::router::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateCommentCommand {
    pub product_id: Uuid,
    pub content: String,
    pub rating: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCommentCommand {
    pub content: Option<String>,
    pub rating: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ListCommentsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CommentView {
    pub id: Uuid,
    pub product_id: Uuid,
    pub user_id: Option<Uuid>,
    pub content: String,
    pub rating: Option<i32>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn create_comment(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Json(cmd): Json<CreateCommentCommand>,
) -> Result<(StatusCode, Json<ApiResponse<Uuid>>), (StatusCode, Json<ApiResponse<()>>)> {
    if cmd.content.trim().is_empty() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ApiResponse::error("content cannot be empty")),
        ));
    }

    if let Some(rating) = cmd.rating {
        if !(1..=5).contains(&rating) {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ApiResponse::error("rating must be between 1 and 5")),
            ));
        }
    }

    ensure_product_exists(&state.product_service_base_url, cmd.product_id).await?;

    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO product_comments
           (id, product_id, user_id, content, rating, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, NOW(), NOW())"#,
    )
    .bind(id)
    .bind(cmd.product_id)
    .bind(Some(user_id))
    .bind(cmd.content.trim())
    .bind(cmd.rating)
    .execute(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok((StatusCode::CREATED, Json(ApiResponse::success(id))))
}

pub async fn get_comment(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<CommentView>>, (StatusCode, Json<ApiResponse<()>>)> {
    let row = sqlx::query_as::<_, CommentView>(
        r#"SELECT id, product_id, user_id, content, rating, created_at, updated_at
           FROM product_comments
           WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?;

    match row {
        Some(view) => Ok(Json(ApiResponse::success(view))),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("comment not found")),
        )),
    }
}

pub async fn list_product_comments(
    State(state): State<AppState>,
    Path(product_id): Path<Uuid>,
    Query(query): Query<ListCommentsQuery>,
) -> Result<Json<ApiResponse<Vec<CommentView>>>, (StatusCode, Json<ApiResponse<()>>)> {
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let offset = query.offset.unwrap_or(0).max(0);

    let rows = sqlx::query_as::<_, CommentView>(
        r#"SELECT id, product_id, user_id, content, rating, created_at, updated_at
           FROM product_comments
           WHERE product_id = $1
           ORDER BY created_at DESC
           LIMIT $2 OFFSET $3"#,
    )
    .bind(product_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(ApiResponse::success(rows)))
}

pub async fn update_comment(
    State(state): State<AppState>,
    Extension(_user_id): Extension<Uuid>,
    Path(id): Path<Uuid>,
    Json(cmd): Json<UpdateCommentCommand>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    let content = cmd.content.map(|c| c.trim().to_string());

    if let Some(ref c) = content {
        if c.is_empty() {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ApiResponse::error("content cannot be empty")),
            ));
        }
    }

    if let Some(rating) = cmd.rating {
        if !(1..=5).contains(&rating) {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ApiResponse::error("rating must be between 1 and 5")),
            ));
        }
    }

    let result = sqlx::query(
        r#"UPDATE product_comments
           SET content = COALESCE($2, content),
               rating = COALESCE($3, rating),
               updated_at = NOW()
           WHERE id = $1"#,
    )
    .bind(id)
    .bind(content)
    .bind(cmd.rating)
    .execute(&state.pool)
    .await
    .map_err(internal_error)?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("comment not found")),
        ));
    }

    Ok(Json(ApiResponse::success(())))
}

pub async fn delete_comment(
    State(state): State<AppState>,
    Extension(_user_id): Extension<Uuid>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    let result = sqlx::query("DELETE FROM product_comments WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("comment not found")),
        ));
    }

    Ok(Json(ApiResponse::success(())))
}

fn internal_error(err: sqlx::Error) -> (StatusCode, Json<ApiResponse<()>>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiResponse::error(format!("database error: {}", err))),
    )
}

async fn ensure_product_exists(
    product_service_base_url: &str,
    product_id: Uuid,
) -> Result<(), (StatusCode, Json<ApiResponse<()>>)> {
    let url = format!(
        "{}/api/v1/products/{}",
        product_service_base_url.trim_end_matches('/'),
        product_id
    );

    let resp = reqwest::Client::new().get(url).send().await.map_err(|_| {
        (
            StatusCode::BAD_GATEWAY,
            Json(ApiResponse::error(
                "Failed to validate product via product-service",
            )),
        )
    })?;

    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ApiResponse::error("product_id does not exist")),
        ));
    }

    if !resp.status().is_success() {
        return Err((
            StatusCode::BAD_GATEWAY,
            Json(ApiResponse::error(
                "Failed to validate product via product-service",
            )),
        ));
    }

    Ok(())
}
