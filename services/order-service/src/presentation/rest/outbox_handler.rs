use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;

use super::response::ApiResponse;
use super::router::AppState;

#[derive(Debug, Deserialize)]
pub struct ListOutboxQuery {
    pub status: Option<String>,
    pub limit: Option<i64>,
}

pub async fn outbox_stats(
    State(state): State<Arc<AppState>>,
) -> Result<
    Json<ApiResponse<crate::domain::entities::OutboxStats>>,
    (StatusCode, Json<ApiResponse<()>>),
> {
    match state.outbox_service.stats().await {
        Ok(stats) => Ok(Json(ApiResponse::success(stats))),
        Err(e) => {
            let (status, msg) = map_domain_error(&e);
            Err((status, Json(ApiResponse::error(msg))))
        }
    }
}

pub async fn list_outbox_messages(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListOutboxQuery>,
) -> Result<
    Json<ApiResponse<Vec<crate::domain::entities::OutboxMessage>>>,
    (StatusCode, Json<ApiResponse<()>>),
> {
    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    match state
        .outbox_service
        .list_messages(query.status.as_deref(), limit)
        .await
    {
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
