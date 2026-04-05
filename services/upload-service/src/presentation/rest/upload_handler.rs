use axum::{extract::State, http::StatusCode, Json};
use std::sync::Arc;

use crate::application::services::{
    CreateUploadPresignRequest, CreateUploadPresignResponse, UploadAppService,
};
use crate::presentation::rest::response::ApiResponse;

pub async fn create_upload_presign_url(
    State(svc): State<Arc<UploadAppService>>,
    Json(req): Json<CreateUploadPresignRequest>,
) -> Result<Json<ApiResponse<CreateUploadPresignResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    match svc.create_presigned_upload_url(req).await {
        Ok(resp) => Ok(Json(ApiResponse::success(resp))),
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
