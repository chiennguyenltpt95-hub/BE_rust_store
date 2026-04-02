use axum::{
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use std::time::Duration;
use tower_http::cors::CorsLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use super::{metrics, shipment_handler};

pub fn build_router(pool: PgPool) -> Router {
    Router::new()
        .route(
            "/api/v1/shipments",
            post(shipment_handler::create_shipment),
        )
        .route(
            "/api/v1/shipments/:id",
            get(shipment_handler::get_shipment),
        )
        .route(
            "/api/v1/shipments/:id/status",
            post(shipment_handler::update_shipment_status),
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
