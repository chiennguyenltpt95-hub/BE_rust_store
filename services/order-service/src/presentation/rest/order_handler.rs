use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use super::router::AppState;
use crate::application::commands::CreateOrderCommand;
use crate::application::queries::OrderView;

use super::response::ApiResponse;

#[derive(Debug, Deserialize)]
pub struct ListOrdersQuery {
    pub user_id: Uuid,
}

pub async fn create_order(
    State(state): State<Arc<AppState>>,
    Json(cmd): Json<CreateOrderCommand>,
) -> Result<(StatusCode, Json<ApiResponse<OrderView>>), (StatusCode, Json<ApiResponse<()>>)> {
    match state.order_service.create_order(cmd).await {
        Ok(view) => Ok((StatusCode::CREATED, Json(ApiResponse::success(view)))),
        Err(e) => {
            let (status, msg) = map_domain_error(&e);
            Err((status, Json(ApiResponse::error(msg))))
        }
    }
}

pub async fn get_order(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<OrderView>>, (StatusCode, Json<ApiResponse<()>>)> {
    match state.order_service.get_order(id).await {
        Ok(view) => Ok(Json(ApiResponse::success(view))),
        Err(e) => {
            let (status, msg) = map_domain_error(&e);
            Err((status, Json(ApiResponse::error(msg))))
        }
    }
}

pub async fn list_orders(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListOrdersQuery>,
) -> Result<Json<ApiResponse<Vec<OrderView>>>, (StatusCode, Json<ApiResponse<()>>)> {
    match state.order_service.list_orders_by_user(query.user_id).await {
        Ok(rows) => Ok(Json(ApiResponse::success(rows))),
        Err(e) => {
            let (status, msg) = map_domain_error(&e);
            Err((status, Json(ApiResponse::error(msg))))
        }
    }
}

fn map_domain_error(err: &domain_core::error::DomainError) -> (StatusCode, String) {
    use domain_core::error::DomainError::*;
    match err {
        NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
        ValidationError(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg.clone()),
        BusinessRuleViolation(msg) => (StatusCode::CONFLICT, msg.clone()),
        Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
        Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
        InfrastructureError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".into()),
    }
}
