use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::CorsLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use crate::application::services::{OrderAppService, OutboxService};

use super::{metrics, order_handler, outbox_handler};

pub struct AppState {
    pub order_service: Arc<OrderAppService>,
    pub outbox_service: Arc<OutboxService>,
}

pub fn build_router(
    order_service: Arc<OrderAppService>,
    outbox_service: Arc<OutboxService>,
) -> Router {
    let state = Arc::new(AppState {
        order_service,
        outbox_service,
    });

    Router::new()
        .route("/api/v1/orders", post(order_handler::create_order))
        .route("/api/v1/orders", get(order_handler::list_orders))
        .route("/api/v1/orders/:id", get(order_handler::get_order))
        .route("/api/v1/outbox/stats", get(outbox_handler::outbox_stats))
        .route(
            "/api/v1/outbox/messages",
            get(outbox_handler::list_outbox_messages),
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
