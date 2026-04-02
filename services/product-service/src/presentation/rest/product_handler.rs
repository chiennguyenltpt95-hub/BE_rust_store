use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::application::commands::{CreateProductCommand, UpdateProductCommand};
use crate::application::queries::{ListProductsQuery, ProductView};
use crate::application::services::ProductAppService;

use super::response::ApiResponse;

pub async fn create_product(
    State(svc): State<Arc<ProductAppService>>,
    Json(cmd): Json<CreateProductCommand>,
) -> Result<(StatusCode, Json<ApiResponse<Uuid>>), (StatusCode, Json<ApiResponse<()>>)> {
    match svc.create_product(cmd).await {
        Ok(id) => Ok((StatusCode::CREATED, Json(ApiResponse::success(id)))),
        Err(e) => {
            let (status, msg) = map_domain_error(&e);
            Err((status, Json(ApiResponse::error(msg))))
        }
    }
}

pub async fn get_product(
    State(svc): State<Arc<ProductAppService>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<ProductView>>, (StatusCode, Json<ApiResponse<()>>)> {
    match svc.get_product(id).await {
        Ok(view) => Ok(Json(ApiResponse::success(view))),
        Err(e) => {
            let (status, msg) = map_domain_error(&e);
            Err((status, Json(ApiResponse::error(msg))))
        }
    }
}

pub async fn list_products(
    State(svc): State<Arc<ProductAppService>>,
    Query(query): Query<ListProductsQuery>,
) -> Result<Json<ApiResponse<Vec<ProductView>>>, (StatusCode, Json<ApiResponse<()>>)> {
    match svc.list_products(query).await {
        Ok(list) => Ok(Json(ApiResponse::success(list))),
        Err(e) => {
            let (status, msg) = map_domain_error(&e);
            Err((status, Json(ApiResponse::error(msg))))
        }
    }
}

pub async fn update_product(
    State(svc): State<Arc<ProductAppService>>,
    Path(id): Path<Uuid>,
    Json(cmd): Json<UpdateProductCommand>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    match svc.update_product(id, cmd).await {
        Ok(_) => Ok(Json(ApiResponse::success(()))),
        Err(e) => {
            let (status, msg) = map_domain_error(&e);
            Err((status, Json(ApiResponse::error(msg))))
        }
    }
}

pub async fn delete_product(
    State(svc): State<Arc<ProductAppService>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    match svc.delete_product(id).await {
        Ok(_) => Ok(Json(ApiResponse::success(()))),
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
