use axum::{routing::get, Router};
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::CorsLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::SwaggerUi;

use super::auth_handler;
use super::user_handler;
use crate::application::commands::auth::{
    LoginCommand, LogoutCommand, RefreshTokenCommand, TokenPair,
};
use crate::application::commands::{CreateUserCommand, UpdateUserCommand};
use crate::application::queries::get_user::UserView;
use crate::application::queries::list_users::UserSummary;
use crate::application::services::auth_service::AuthAppService;
use crate::application::services::UserAppService;

// ---------------------------------------------------------------------------
// OpenAPI manifest — chỉ cần metadata + schemas.
// Paths được tự động collect từ routes!() macro bên dưới (không cần liệt kê).
// ---------------------------------------------------------------------------
#[derive(OpenApi)]
#[openapi(
    info(
        title = "User Service API",
        version = "1.0.0",
        description = "REST API cho User Service — DDD + Microservices"
    ),
    components(schemas(
        CreateUserCommand,
        UpdateUserCommand,
        UserView,
        UserSummary,
        LoginCommand,
        RefreshTokenCommand,
        LogoutCommand,
        TokenPair,
    )),
    tags(
        (name = "Users", description = "CRUD operations cho User"),
        (name = "Auth",  description = "Login, Refresh token, Logout"),
    )
)]
struct ApiDoc;

// ---------------------------------------------------------------------------
// Router — mỗi routes!() tự đăng ký utoipa::path annotations lên OpenApi
// ---------------------------------------------------------------------------
pub fn build_router(
    user_service: Arc<UserAppService>,
    auth_service: Arc<AuthAppService>,
) -> Router {
    // split_for_parts() TRƯỚC with_state() — with_state() trả về Router, không phải OpenApiRouter
    let (user_router, user_api) = OpenApiRouter::new()
        .routes(routes!(user_handler::create_user, user_handler::list_users))
        .routes(routes!(
            user_handler::get_user,
            user_handler::update_user,
            user_handler::delete_user
        ))
        .split_for_parts();

    let (auth_router, auth_api) = OpenApiRouter::new()
        .routes(routes!(auth_handler::login))
        .routes(routes!(auth_handler::refresh))
        .routes(routes!(auth_handler::logout))
        .split_for_parts();

    // Merge tất cả paths vào một OpenApi document, thêm prefix cho đúng route thực tế
    let mut api = ApiDoc::openapi();

    let mut user_api = user_api;
    let user_paths: Vec<_> = user_api.paths.paths.keys().cloned().collect();
    for old_path in user_paths {
        if let Some(item) = user_api.paths.paths.remove(&old_path) {
            let new_path = format!("/api/v1/users{old_path}");
            let new_path = new_path.trim_end_matches('/').to_string();
            let new_path = if new_path.is_empty() {
                "/".to_string()
            } else {
                new_path
            };
            user_api.paths.paths.insert(new_path, item);
        }
    }

    let mut auth_api = auth_api;
    let auth_paths: Vec<_> = auth_api.paths.paths.keys().cloned().collect();
    for old_path in auth_paths {
        if let Some(item) = auth_api.paths.paths.remove(&old_path) {
            let new_path = format!("/api/v1/auth{old_path}");
            let new_path = new_path.trim_end_matches('/').to_string();
            let new_path = if new_path.is_empty() {
                "/".to_string()
            } else {
                new_path
            };
            auth_api.paths.paths.insert(new_path, item);
        }
    }

    api.merge(user_api);
    api.merge(auth_api);

    Router::new()
        .nest("/api/v1/users", user_router.with_state(user_service))
        .nest("/api/v1/auth", auth_router.with_state(auth_service))
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api))
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(CorsLayer::permissive())
}

async fn health_check() -> &'static str {
    "OK"
}

async fn readiness_check() -> &'static str {
    "READY"
}
