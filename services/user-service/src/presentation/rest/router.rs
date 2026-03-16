use axum::{
    Router,
    routing::{get, post, put, delete},
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tower_http::timeout::TimeoutLayer;
use std::time::Duration;

use crate::application::services::UserAppService;
use super::user_handler;

pub fn build_router(user_service: Arc<UserAppService>) -> Router {
    let user_routes = Router::new()
        .route("/", post(user_handler::create_user))
        .route("/", get(user_handler::list_users))
        .route("/:id", get(user_handler::get_user))
        .route("/:id", put(user_handler::update_user))
        .route("/:id", delete(user_handler::delete_user))
        .with_state(user_service);

    Router::new()
        .nest("/api/v1/users", user_routes)
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
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
