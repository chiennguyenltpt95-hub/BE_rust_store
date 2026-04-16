use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

use crate::application::commands::CreateCategoryCommand;
use crate::application::queries::CategoryView;
use crate::application::services::CategoryAppService;

use super::response::ApiResponse;

pub async fn create_category(
    State(svc): State<Arc<CategoryAppService>>,
    Json(cmd): Json<CreateCategoryCommand>,
) -> Result<(StatusCode, Json<ApiResponse<CategoryView>>), (StatusCode, Json<ApiResponse<()>>)> {
    match svc.create_category(cmd).await {
        Ok(category_view) => Ok((
            StatusCode::CREATED,
            Json(ApiResponse::success(category_view)),
        )),
        Err(e) => {
            let (status, msg) = map_domain_error(&e);
            Err((status, Json(ApiResponse::error(msg))))
        }
    }
}

pub async fn get_category(
    State(svc): State<Arc<CategoryAppService>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<CategoryView>>, (StatusCode, Json<ApiResponse<()>>)> {
    match svc.get_category(id).await {
        Ok(view) => Ok(Json(ApiResponse::success(view))),
        Err(e) => {
            let (status, msg) = map_domain_error(&e);
            Err((status, Json(ApiResponse::error(msg))))
        }
    }
}

pub async fn list_categories(
    State(svc): State<Arc<CategoryAppService>>,
) -> Result<Json<ApiResponse<Vec<CategoryView>>>, (StatusCode, Json<ApiResponse<()>>)> {
    match svc.list_categories().await {
        Ok(list) => Ok(Json(ApiResponse::success(list))),
        Err(e) => {
            let (status, msg) = map_domain_error(&e);
            Err((status, Json(ApiResponse::error(msg))))
        }
    }
}

pub async fn delete_category(
    State(svc): State<Arc<CategoryAppService>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    match svc.delete_category(id).await {
        Ok(_) => Ok(Json(ApiResponse::success(()))),
        Err(e) => {
            let (status, msg) = map_domain_error(&e);
            Err((status, Json(ApiResponse::error(msg))))
        }
    }
}

pub async fn search_categories(
    State(svc): State<Arc<CategoryAppService>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<Vec<CategoryView>>>, (StatusCode, Json<ApiResponse<()>>)> {
    let query = params.get("query").cloned().unwrap_or_default();
    match svc.search_categories(query).await {
        Ok(list) => Ok(Json(ApiResponse::success(list))),
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
