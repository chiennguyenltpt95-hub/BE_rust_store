use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::CorsLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use crate::application::services::CartAppService;

use super::{cart_handler, metrics};

pub fn build_router(cart_service: Arc<CartAppService>) -> Router {
    Router::new()
        .route("/api/v1/carts", post(cart_handler::create_cart))
        .route("/api/v1/carts/:id", get(cart_handler::get_cart))
        .route(
            "/api/v1/carts/active/:user_id",
            get(cart_handler::get_active_cart_by_user),
        )
        .route("/api/v1/carts/:id/items", post(cart_handler::add_item))
        .route(
            "/api/v1/carts/:id/items/:item_id",
            put(cart_handler::update_item_quantity),
        )
        .route(
            "/api/v1/carts/:id/items/:item_id",
            delete(cart_handler::remove_item),
        )
        .route(
            "/api/v1/carts/:id/checkout",
            post(cart_handler::checkout_cart),
        )
        .route("/metrics", get(metrics::metrics))
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
        .with_state(cart_service)
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
