use axum::{
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use std::time::Duration;
use tower_http::cors::CorsLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use super::{inventory_handler, metrics};

pub fn build_router(pool: PgPool) -> Router {
    Router::new()
        .route(
            "/api/v1/inventory/:product_id",
            get(inventory_handler::get_inventory),
        )
        .route(
            "/api/v1/inventory/reserve",
            post(inventory_handler::reserve_inventory),
        )
        .route(
            "/api/v1/inventory/release",
            post(inventory_handler::release_inventory),
        )
        .route("/metrics", get(metrics::metrics))
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
        .with_state(pool)
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
