use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::application::commands::{AddItemCommand, CreateCartCommand, UpdateItemQuantityCommand};
use crate::application::queries::CartView;
use crate::application::services::CartAppService;

use super::response::ApiResponse;

pub async fn create_cart(
    State(svc): State<Arc<CartAppService>>,
    Json(cmd): Json<CreateCartCommand>,
) -> Result<(StatusCode, Json<ApiResponse<Uuid>>), (StatusCode, Json<ApiResponse<()>>)> {
    match svc.create_cart(cmd).await {
        Ok(id) => Ok((StatusCode::CREATED, Json(ApiResponse::success(id)))),
        Err(e) => {
            let (status, msg) = map_domain_error(&e);
            Err((status, Json(ApiResponse::error(msg))))
        }
    }
}

pub async fn get_cart(
    State(svc): State<Arc<CartAppService>>,
    Path(cart_id): Path<Uuid>,
) -> Result<Json<ApiResponse<CartView>>, (StatusCode, Json<ApiResponse<()>>)> {
    match svc.get_cart(cart_id).await {
        Ok(view) => Ok(Json(ApiResponse::success(view))),
        Err(e) => {
            let (status, msg) = map_domain_error(&e);
            Err((status, Json(ApiResponse::error(msg))))
        }
    }
}

pub async fn get_active_cart_by_user(
    State(svc): State<Arc<CartAppService>>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<ApiResponse<CartView>>, (StatusCode, Json<ApiResponse<()>>)> {
    match svc.get_active_cart_by_user(user_id).await {
        Ok(view) => Ok(Json(ApiResponse::success(view))),
        Err(e) => {
            let (status, msg) = map_domain_error(&e);
            Err((status, Json(ApiResponse::error(msg))))
        }
    }
}

pub async fn add_item(
    State(svc): State<Arc<CartAppService>>,
    Path(cart_id): Path<Uuid>,
    Json(cmd): Json<AddItemCommand>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    match svc.add_item(cart_id, cmd).await {
        Ok(_) => Ok(Json(ApiResponse::success(()))),
        Err(e) => {
            let (status, msg) = map_domain_error(&e);
            Err((status, Json(ApiResponse::error(msg))))
        }
    }
}

pub async fn update_item_quantity(
    State(svc): State<Arc<CartAppService>>,
    Path((cart_id, item_id)): Path<(Uuid, Uuid)>,
    Json(cmd): Json<UpdateItemQuantityCommand>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    match svc.update_item_quantity(cart_id, item_id, cmd).await {
        Ok(_) => Ok(Json(ApiResponse::success(()))),
        Err(e) => {
            let (status, msg) = map_domain_error(&e);
            Err((status, Json(ApiResponse::error(msg))))
        }
    }
}

pub async fn remove_item(
    State(svc): State<Arc<CartAppService>>,
    Path((cart_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    match svc.remove_item(cart_id, item_id).await {
        Ok(_) => Ok(Json(ApiResponse::success(()))),
        Err(e) => {
            let (status, msg) = map_domain_error(&e);
            Err((status, Json(ApiResponse::error(msg))))
        }
    }
}

pub async fn checkout_cart(
    State(svc): State<Arc<CartAppService>>,
    Path(cart_id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    match svc.checkout_cart(cart_id).await {
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
