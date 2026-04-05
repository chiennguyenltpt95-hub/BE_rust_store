use axum::extract::Request;
use axum::http::{header, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use axum::Json;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::Deserialize;

use crate::presentation::rest::response::ApiResponse;

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
struct Claims {
    sub: String,
    payload: serde_json::Value,
    exp: i64,
    iat: i64,
}

pub async fn require_admin(
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ApiResponse<()>>)> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();

    let token = auth_header
        .strip_prefix("Bearer ")
        .or_else(|| auth_header.strip_prefix("bearer "))
        .unwrap_or_default()
        .trim();

    if token.is_empty() {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::error("Missing or invalid Authorization header")),
        ));
    }

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "super-secret-change-me".into());
    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map(|d| d.claims)
    .map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::error("Unauthorized: Invalid token")),
        )
    })?;

    let role = claims
        .payload
        .get("role")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_lowercase();

    if role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Forbidden: Admin role required")),
        ));
    }

    Ok(next.run(request).await)
}
