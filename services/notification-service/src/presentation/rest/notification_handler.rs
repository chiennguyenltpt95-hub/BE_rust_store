use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use super::response::ApiResponse;
use super::router::AppState;

#[derive(Debug, Deserialize)]
pub struct SendNotificationCommand {
    pub channel: String,
    pub recipient: String,
    pub template_name: Option<String>,
    pub payload: serde_json::Value,
    pub max_attempts: Option<i32>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct NotificationView {
    pub id: Uuid,
    pub channel: String,
    pub recipient: String,
    pub template_name: Option<String>,
    pub payload: serde_json::Value,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn send_notification(
    State(state): State<Arc<AppState>>,
    Json(cmd): Json<SendNotificationCommand>,
) -> Result<(StatusCode, Json<ApiResponse<Uuid>>), (StatusCode, Json<ApiResponse<()>>)> {
    let channel = cmd.channel.to_lowercase();
    if !matches!(channel.as_str(), "telegram" | "email" | "sms" | "push" | "webhook") {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ApiResponse::error(
                "channel must be one of: telegram, email, sms, push, webhook",
            )),
        ));
    }

    if cmd.recipient.trim().is_empty() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ApiResponse::error("recipient cannot be empty")),
        ));
    }

    let max_attempts = cmd
        .max_attempts
        .unwrap_or(state.default_max_attempts)
        .max(1);

    let id: Uuid = sqlx::query_scalar(
        r#"INSERT INTO notifications
           (id, channel, recipient, template_name, payload, status, attempts, max_attempts, next_retry_at, created_at, updated_at)
           VALUES (gen_random_uuid(), $1, $2, $3, $4, 'queued', 0, $5, NOW(), NOW(), NOW())
           RETURNING id"#,
    )
    .bind(&channel)
    .bind(&cmd.recipient)
    .bind(&cmd.template_name)
    .bind(&cmd.payload)
    .bind(max_attempts)
    .fetch_one(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok((StatusCode::ACCEPTED, Json(ApiResponse::success(id))))
}

pub async fn get_notification(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<NotificationView>>, (StatusCode, Json<ApiResponse<()>>)> {
    let row = sqlx::query_as::<_, NotificationView>(
        r#"SELECT id, channel, recipient, template_name, payload, status, created_at
           FROM notifications
           WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?;

    match row {
        Some(view) => Ok(Json(ApiResponse::success(view))),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("notification not found")),
        )),
    }
}

fn internal_error(err: sqlx::Error) -> (StatusCode, Json<ApiResponse<()>>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiResponse::error(format!("database error: {}", err))),
    )
}
