use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::CorsLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use crate::application::services::CheckoutAppService;

use super::{checkout_handler, metrics};

pub fn build_router(service: Arc<CheckoutAppService>) -> Router {
    Router::new()
        .route("/api/v1/checkouts", post(checkout_handler::create_checkout))
        .route("/api/v1/checkouts/:id", get(checkout_handler::get_checkout))
        .route(
            "/api/v1/checkouts/:id/mark-paid",
            post(checkout_handler::mark_paid),
        )
        .route(
            "/api/v1/checkouts/:id/mark-failed",
            post(checkout_handler::mark_failed),
        )
        .route(
            "/api/v1/checkouts/webhooks/paypal",
            post(checkout_handler::paypal_webhook),
        )
        .route(
            "/api/v1/checkouts/webhooks/stripe",
            post(checkout_handler::stripe_webhook),
        )
        .route(
            "/api/v1/checkouts/webhooks/oxapay",
            post(checkout_handler::oxapay_webhook),
        )
        .route("/metrics", get(metrics::metrics))
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
        .with_state(service)
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
