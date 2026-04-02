use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use super::response::ApiResponse;

#[derive(Debug, Deserialize)]
pub struct CreateShipmentCommand {
    pub order_id: Uuid,
    pub shipping_address: String,
    pub carrier: Option<String>,
    pub tracking_code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateShipmentStatusCommand {
    pub status: String,
    pub carrier: Option<String>,
    pub tracking_code: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ShipmentView {
    pub id: Uuid,
    pub order_id: Uuid,
    pub carrier: Option<String>,
    pub tracking_code: Option<String>,
    pub status: String,
    pub shipping_address: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn create_shipment(
    State(pool): State<PgPool>,
    Json(cmd): Json<CreateShipmentCommand>,
) -> Result<(StatusCode, Json<ApiResponse<Uuid>>), (StatusCode, Json<ApiResponse<()>>)> {
    if cmd.shipping_address.trim().is_empty() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ApiResponse::error("shipping_address cannot be empty")),
        ));
    }

    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO shipments
           (id, order_id, carrier, tracking_code, status, shipping_address, created_at, updated_at)
           VALUES ($1, $2, $3, $4, 'pending', $5, NOW(), NOW())"#,
    )
    .bind(id)
    .bind(cmd.order_id)
    .bind(cmd.carrier)
    .bind(cmd.tracking_code)
    .bind(cmd.shipping_address)
    .execute(&pool)
    .await
    .map_err(internal_error)?;

    Ok((StatusCode::CREATED, Json(ApiResponse::success(id))))
}

pub async fn get_shipment(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<ShipmentView>>, (StatusCode, Json<ApiResponse<()>>)> {
    let row = sqlx::query_as::<_, ShipmentView>(
        r#"SELECT id, order_id, carrier, tracking_code, status, shipping_address, created_at, updated_at
           FROM shipments WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(internal_error)?;

    match row {
        Some(view) => Ok(Json(ApiResponse::success(view))),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("shipment not found")),
        )),
    }
}

pub async fn update_shipment_status(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(cmd): Json<UpdateShipmentStatusCommand>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    let status = cmd.status.to_lowercase();
    if !matches!(
        status.as_str(),
        "pending" | "packed" | "shipped" | "delivered" | "failed"
    ) {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ApiResponse::error(
                "status must be one of: pending, packed, shipped, delivered, failed",
            )),
        ));
    }

    let result = sqlx::query(
        r#"UPDATE shipments
           SET status = $2,
               carrier = COALESCE($3, carrier),
               tracking_code = COALESCE($4, tracking_code),
               updated_at = NOW()
           WHERE id = $1"#,
    )
    .bind(id)
    .bind(status)
    .bind(cmd.carrier)
    .bind(cmd.tracking_code)
    .execute(&pool)
    .await
    .map_err(internal_error)?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("shipment not found")),
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
