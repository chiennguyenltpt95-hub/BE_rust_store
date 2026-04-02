use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::application::commands::{
    CreateCheckoutCommand, MarkFailedCommand, MarkPaidCommand,
};
use crate::application::queries::CheckoutView;
use crate::application::services::CheckoutAppService;

use super::response::ApiResponse;

pub async fn create_checkout(
    State(svc): State<Arc<CheckoutAppService>>,
    Json(cmd): Json<CreateCheckoutCommand>,
) -> Result<(StatusCode, Json<ApiResponse<CheckoutView>>), (StatusCode, Json<ApiResponse<()>>)> {
    match svc.create_checkout(cmd).await {
        Ok(view) => Ok((StatusCode::CREATED, Json(ApiResponse::success(view)))),
        Err(e) => {
            let (status, msg) = map_domain_error(&e);
            Err((status, Json(ApiResponse::error(msg))))
        }
    }
}

pub async fn get_checkout(
    State(svc): State<Arc<CheckoutAppService>>,
    Path(checkout_id): Path<Uuid>,
) -> Result<Json<ApiResponse<CheckoutView>>, (StatusCode, Json<ApiResponse<()>>)> {
    match svc.get_checkout(checkout_id).await {
        Ok(view) => Ok(Json(ApiResponse::success(view))),
        Err(e) => {
            let (status, msg) = map_domain_error(&e);
            Err((status, Json(ApiResponse::error(msg))))
        }
    }
}

pub async fn mark_paid(
    State(svc): State<Arc<CheckoutAppService>>,
    Path(checkout_id): Path<Uuid>,
    Json(cmd): Json<MarkPaidCommand>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    match svc.mark_paid(checkout_id, cmd).await {
        Ok(_) => Ok(Json(ApiResponse::success(()))),
        Err(e) => {
            let (status, msg) = map_domain_error(&e);
            Err((status, Json(ApiResponse::error(msg))))
        }
    }
}

pub async fn mark_failed(
    State(svc): State<Arc<CheckoutAppService>>,
    Path(checkout_id): Path<Uuid>,
    Json(cmd): Json<MarkFailedCommand>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    match svc.mark_failed(checkout_id, cmd).await {
        Ok(_) => Ok(Json(ApiResponse::success(()))),
        Err(e) => {
            let (status, msg) = map_domain_error(&e);
            Err((status, Json(ApiResponse::error(msg))))
        }
    }
}

pub async fn paypal_webhook(
    State(svc): State<Arc<CheckoutAppService>>,
    headers: HeaderMap,
    payload: Bytes,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    let signature = extract_signature(
        &headers,
        &["paypal-transmission-sig", "x-paypal-signature", "x-signature"],
    )?;

    match svc
        .handle_provider_webhook_signed("paypal", &signature, &payload)
        .await
    {
        Ok(_) => Ok(Json(ApiResponse::success(()))),
        Err(e) => {
            let (status, msg) = map_domain_error(&e);
            Err((status, Json(ApiResponse::error(msg))))
        }
    }
}

pub async fn stripe_webhook(
    State(svc): State<Arc<CheckoutAppService>>,
    headers: HeaderMap,
    payload: Bytes,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    let signature = extract_signature(&headers, &["stripe-signature", "x-signature"])?;

    match svc
        .handle_provider_webhook_signed("stripe", &signature, &payload)
        .await
    {
        Ok(_) => Ok(Json(ApiResponse::success(()))),
        Err(e) => {
            let (status, msg) = map_domain_error(&e);
            Err((status, Json(ApiResponse::error(msg))))
        }
    }
}

pub async fn oxapay_webhook(
    State(svc): State<Arc<CheckoutAppService>>,
    headers: HeaderMap,
    payload: Bytes,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    let signature = extract_signature(&headers, &["x-oxapay-signature", "x-signature"])?;

    match svc
        .handle_provider_webhook_signed("oxapay", &signature, &payload)
        .await
    {
        Ok(_) => Ok(Json(ApiResponse::success(()))),
        Err(e) => {
            let (status, msg) = map_domain_error(&e);
            Err((status, Json(ApiResponse::error(msg))))
        }
    }
}

fn extract_signature(
    headers: &HeaderMap,
    candidates: &[&str],
) -> Result<String, (StatusCode, Json<ApiResponse<()>>)> {
    for key in candidates {
        if let Some(value) = headers.get(*key) {
            if let Ok(sig) = value.to_str() {
                let trimmed = sig.trim();
                if !trimmed.is_empty() {
                    return Ok(trimmed.to_string());
                }
            }
        }
    }

    Err((
        StatusCode::UNAUTHORIZED,
        Json(ApiResponse::error("Missing webhook signature header")),
    ))
}

fn map_domain_error(err: &domain_core::error::DomainError) -> (StatusCode, String) {
    use domain_core::error::DomainError::*;
    match err {
        NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
        ValidationError(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg.clone()),
        BusinessRuleViolation(msg) => (StatusCode::CONFLICT, msg.clone()),
        Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
        Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
        InfrastructureError(msg) => (StatusCode::BAD_GATEWAY, msg.clone()),
    }
}
