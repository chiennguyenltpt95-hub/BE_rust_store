use axum::extract::Request;
use axum::http::{header, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use axum::Json;
use uuid::Uuid;

use crate::infrastructure::auth::JwtService;
use crate::presentation::rest::response::ApiResponse;

pub async fn require_jwt(
    mut request: Request,
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

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "super-secret-change-me".to_string());
    let jwt = JwtService::new(&secret);

    let claims = if let Ok(claims) = jwt.verify_access_token(token) {
        claims
    } else {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::error("Unauthorized: Invalid token")),
        ));
    };

    let user_id = if let Ok(id) = Uuid::parse_str(&claims.sub) {
        id
    } else {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::error("Unauthorized: Invalid token subject")),
        ));
    };

    request.extensions_mut().insert(user_id);

    Ok(next.run(request).await)
}
