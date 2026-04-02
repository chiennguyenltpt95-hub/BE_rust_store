use axum::{
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::CorsLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use super::{metrics, notification_handler};

pub struct AppState {
    pub pool: PgPool,
    pub default_max_attempts: i32,
}

pub fn build_router(
    pool: PgPool,
    default_max_attempts: i32,
) -> Router {
    let state = Arc::new(AppState {
        pool,
        default_max_attempts,
    });

    Router::new()
        .route(
            "/api/v1/notifications/send",
            post(notification_handler::send_notification),
        )
        .route(
            "/api/v1/notifications/:id",
            get(notification_handler::get_notification),
        )
        .route("/metrics", get(metrics::metrics))
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
        .with_state(state)
        .layer(axum::middleware::from_fn(metrics::track_metrics))
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
