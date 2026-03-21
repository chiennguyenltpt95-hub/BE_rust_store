use axum::{extract::State, http::StatusCode, Json};
use std::sync::Arc;

use crate::application::commands::auth::{
    LoginCommand, LogoutCommand, RefreshTokenCommand, TokenPair,
};
use crate::application::services::auth_service::AuthAppService;
use crate::presentation::rest::response::ApiResponse;

/// POST /api/v1/auth/login
#[utoipa::path(
    post,
    path = "/login",
    tag = "Auth",
    request_body = LoginCommand,
    responses(
        (status = 200, description = "Login successful", body = TokenPair),
        (status = 401, description = "Invalid credentials"),
        (status = 422, description = "Validation error"),
    )
)]
pub async fn login(
    State(svc): State<Arc<AuthAppService>>,
    Json(cmd): Json<LoginCommand>,
) -> Result<Json<ApiResponse<TokenPair>>, (StatusCode, Json<ApiResponse<()>>)> {
    match svc.login(cmd).await {
        Ok(pair) => Ok(Json(ApiResponse::success(pair))),
        Err(e) => Err(map_err(&e)),
    }
}

/// POST /api/v1/auth/refresh
#[utoipa::path(
    post,
    path = "/refresh",
    tag = "Auth",
    request_body = RefreshTokenCommand,
    responses(
        (status = 200, description = "New token pair", body = TokenPair),
        (status = 401, description = "Token expired or revoked"),
    )
)]
pub async fn refresh(
    State(svc): State<Arc<AuthAppService>>,
    Json(cmd): Json<RefreshTokenCommand>,
) -> Result<Json<ApiResponse<TokenPair>>, (StatusCode, Json<ApiResponse<()>>)> {
    match svc.refresh(cmd).await {
        Ok(pair) => Ok(Json(ApiResponse::success(pair))),
        Err(e) => Err(map_err(&e)),
    }
}

/// POST /api/v1/auth/logout
#[utoipa::path(
    post,
    path = "/logout",
    tag = "Auth",
    request_body = LogoutCommand,
    responses(
        (status = 200, description = "Logged out"),
        (status = 401, description = "Invalid token"),
    )
)]
pub async fn logout(
    State(svc): State<Arc<AuthAppService>>,
    Json(cmd): Json<LogoutCommand>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    match svc.logout(cmd).await {
        Ok(_) => Ok(Json(ApiResponse::success(()))),
        Err(e) => Err(map_err(&e)),
    }
}

fn map_err(err: &domain_core::error::DomainError) -> (StatusCode, Json<ApiResponse<()>>) {
    use domain_core::error::DomainError::*;
    let (status, msg) = match err {
        Unauthorized(m) => (StatusCode::UNAUTHORIZED, m.clone()),
        ValidationError(m) => (StatusCode::UNPROCESSABLE_ENTITY, m.clone()),
        NotFound(m) => (StatusCode::NOT_FOUND, m.clone()),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".into()),
    };
    (status, Json(ApiResponse::error(msg)))
}
