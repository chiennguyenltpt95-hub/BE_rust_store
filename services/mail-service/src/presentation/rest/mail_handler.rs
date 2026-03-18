use axum::{extract::State, http::StatusCode, Json};
use std::sync::Arc;

use crate::application::commands::{SendRawMailCommand, SendTemplatedMailCommand};
use crate::application::services::MailAppService;
use super::response::ApiResponse;

/// POST /api/v1/mail/send
#[utoipa::path(
    post,
    path = "/api/v1/mail/send",
    tag = "Mail",
    request_body = SendRawMailCommand,
    responses(
        (status = 200, description = "Email sent"),
        (status = 422, description = "Validation error"),
        (status = 500, description = "Send failed"),
    )
)]
pub async fn send_raw_mail(
    State(svc): State<Arc<MailAppService>>,
    Json(cmd): Json<SendRawMailCommand>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    match svc.send_raw_mail(cmd).await {
        Ok(_) => Ok(Json(ApiResponse::ok())),
        Err(e) => {
            let (status, msg) = map_domain_error(&e);
            Err((status, Json(ApiResponse::error(msg))))
        }
    }
}

/// POST /api/v1/mail/send-template
#[utoipa::path(
    post,
    path = "/api/v1/mail/send-template",
    tag = "Mail",
    request_body = SendTemplatedMailCommand,
    responses(
        (status = 200, description = "Templated email sent"),
        (status = 422, description = "Validation / template error"),
        (status = 500, description = "Send failed"),
    )
)]
pub async fn send_templated_mail(
    State(svc): State<Arc<MailAppService>>,
    Json(cmd): Json<SendTemplatedMailCommand>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    match svc.send_templated_mail(cmd).await {
        Ok(_) => Ok(Json(ApiResponse::ok())),
        Err(e) => {
            let (status, msg) = map_domain_error(&e);
            Err((status, Json(ApiResponse::error(msg))))
        }
    }
}

fn map_domain_error(err: &domain_core::error::DomainError) -> (StatusCode, String) {
    use domain_core::error::DomainError::*;
    match err {
        ValidationError(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg.clone()),
        InfrastructureError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Mail send failed".into()),
        other => (StatusCode::INTERNAL_SERVER_ERROR, other.to_string()),
    }
}
