use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::application::commands::{CreateUserCommand, DeleteUserCommand, UpdateUserCommand};
use crate::application::queries::get_user::UserView;
use crate::application::queries::list_users::{ListUsersQuery, UserSummary};
use crate::application::services::UserAppService;
use super::response::ApiResponse;

/// POST /api/v1/users
#[utoipa::path(
    post,
    path = "/api/v1/users",
    tag = "Users",
    request_body = CreateUserCommand,
    responses(
        (status = 201, description = "User created", body = Uuid),
        (status = 409, description = "Email already exists"),
        (status = 422, description = "Validation error"),
    )
)]
pub async fn create_user(
    State(svc): State<Arc<UserAppService>>,
    Json(cmd): Json<CreateUserCommand>,
) -> Result<(StatusCode, Json<ApiResponse<Uuid>>), (StatusCode, Json<ApiResponse<()>>)> {
    match svc.create_user(cmd).await {
        Ok(id) => Ok((StatusCode::CREATED, Json(ApiResponse::success(id)))),
        Err(e) => {
            let (status, msg) = map_domain_error(&e);
            Err((status, Json(ApiResponse::error(msg))))
        }
    }
}

/// GET /api/v1/users/:id
#[utoipa::path(
    get,
    path = "/api/v1/users/{id}",
    tag = "Users",
    params(("id" = Uuid, Path, description = "User ID")),
    responses(
        (status = 200, description = "User found", body = UserView),
        (status = 404, description = "User not found"),
    )
)]
pub async fn get_user(
    State(svc): State<Arc<UserAppService>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<UserView>>, (StatusCode, Json<ApiResponse<()>>)> {
    match svc.get_user(id).await {
        Ok(view) => Ok(Json(ApiResponse::success(view))),
        Err(e) => {
            let (status, msg) = map_domain_error(&e);
            Err((status, Json(ApiResponse::error(msg))))
        }
    }
}

/// GET /api/v1/users
#[utoipa::path(
    get,
    path = "/api/v1/users",
    tag = "Users",
    params(ListUsersQuery),
    responses(
        (status = 200, description = "List of users", body = inline(Vec<UserSummary>)),
    )
)]
pub async fn list_users(
    State(svc): State<Arc<UserAppService>>,
    Query(query): Query<ListUsersQuery>,
) -> Result<Json<ApiResponse<Vec<UserSummary>>>, (StatusCode, Json<ApiResponse<()>>)> {
    match svc.list_users(query).await {
        Ok(list) => Ok(Json(ApiResponse::success(list))),
        Err(e) => {
            let (status, msg) = map_domain_error(&e);
            Err((status, Json(ApiResponse::error(msg))))
        }
    }
}

/// PUT /api/v1/users/:id
#[utoipa::path(
    put,
    path = "/api/v1/users/{id}",
    tag = "Users",
    params(("id" = Uuid, Path, description = "User ID")),
    request_body = UpdateUserCommand,
    responses(
        (status = 200, description = "User updated"),
        (status = 404, description = "User not found"),
    )
)]
pub async fn update_user(
    State(svc): State<Arc<UserAppService>>,
    Path(id): Path<Uuid>,
    Json(mut cmd): Json<UpdateUserCommand>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    cmd.user_id = id;
    match svc.update_user(cmd).await {
        Ok(_) => Ok(Json(ApiResponse::success(()))),
        Err(e) => {
            let (status, msg) = map_domain_error(&e);
            Err((status, Json(ApiResponse::error(msg))))
        }
    }
}

/// DELETE /api/v1/users/:id
#[utoipa::path(
    delete,
    path = "/api/v1/users/{id}",
    tag = "Users",
    params(("id" = Uuid, Path, description = "User ID")),
    responses(
        (status = 200, description = "User deleted"),
        (status = 404, description = "User not found"),
    )
)]
pub async fn delete_user(
    State(svc): State<Arc<UserAppService>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    match svc.delete_user(DeleteUserCommand { user_id: id }).await {
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
