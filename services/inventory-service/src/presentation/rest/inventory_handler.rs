use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use super::response::ApiResponse;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct InventoryView {
    pub product_id: Uuid,
    pub available_qty: i32,
    pub reserved_qty: i32,
}

#[derive(Debug, Deserialize)]
pub struct InventoryAdjustCommand {
    pub product_id: Uuid,
    pub quantity: i32,
}

pub async fn get_inventory(
    State(pool): State<PgPool>,
    Path(product_id): Path<Uuid>,
) -> Result<Json<ApiResponse<InventoryView>>, (StatusCode, Json<ApiResponse<()>>)> {
    let row = sqlx::query_as::<_, InventoryView>(
        "SELECT product_id, available_qty, reserved_qty FROM inventory_items WHERE product_id = $1",
    )
    .bind(product_id)
    .fetch_optional(&pool)
    .await
    .map_err(internal_error)?;

    match row {
        Some(view) => Ok(Json(ApiResponse::success(view))),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Inventory not found")),
        )),
    }
}

pub async fn reserve_inventory(
    State(pool): State<PgPool>,
    Json(cmd): Json<InventoryAdjustCommand>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    if cmd.quantity <= 0 {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ApiResponse::error("quantity must be > 0")),
        ));
    }

    let result = sqlx::query(
        r#"UPDATE inventory_items
           SET available_qty = available_qty - $2,
               reserved_qty = reserved_qty + $2,
               updated_at = NOW()
           WHERE product_id = $1 AND available_qty >= $2"#,
    )
    .bind(cmd.product_id)
    .bind(cmd.quantity)
    .execute(&pool)
    .await
    .map_err(internal_error)?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::CONFLICT,
            Json(ApiResponse::error("insufficient inventory or product not found")),
        ));
    }

    Ok(Json(ApiResponse::success(())))
}

pub async fn release_inventory(
    State(pool): State<PgPool>,
    Json(cmd): Json<InventoryAdjustCommand>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    if cmd.quantity <= 0 {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ApiResponse::error("quantity must be > 0")),
        ));
    }

    let result = sqlx::query(
        r#"UPDATE inventory_items
           SET available_qty = available_qty + $2,
               reserved_qty = reserved_qty - $2,
               updated_at = NOW()
           WHERE product_id = $1 AND reserved_qty >= $2"#,
    )
    .bind(cmd.product_id)
    .bind(cmd.quantity)
    .execute(&pool)
    .await
    .map_err(internal_error)?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::CONFLICT,
            Json(ApiResponse::error("reserved quantity is not enough or product not found")),
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
