use axum::{middleware, routing::get, routing::post, Router};
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::CorsLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use crate::application::services::UploadAppService;

use super::{jwt_auth, upload_handler};

pub fn build_router(upload_service: Arc<UploadAppService>) -> Router {
    let upload_admin_router = Router::new()
        .route(
            "/api/v1/uploads/presign",
            post(upload_handler::create_upload_presign_url),
        )
        .with_state(upload_service)
        .route_layer(middleware::from_fn(jwt_auth::require_admin));

    Router::new()
        .merge(upload_admin_router)
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
