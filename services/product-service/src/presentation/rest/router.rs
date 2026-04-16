use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::CorsLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use crate::application::services::{CategoryAppService, ProductAppService};

use super::{category_handler, jwt_auth, metrics, product_handler};

pub fn build_router(
    product_service: Arc<ProductAppService>,
    category_service: Arc<CategoryAppService>,
) -> Router {
    let product_public_router = Router::new()
        .route("/api/v1/products", get(product_handler::list_products))
        .route("/api/v1/products/:id", get(product_handler::get_product))
        .with_state(product_service.clone());

    let product_admin_router = Router::new()
        .route("/api/v1/products", post(product_handler::create_product))
        .route("/api/v1/products/:id", put(product_handler::update_product))
        .route(
            "/api/v1/products/:id",
            delete(product_handler::delete_product),
        )
        .with_state(product_service)
        .route_layer(middleware::from_fn(jwt_auth::require_admin));

    let category_public_router = Router::new()
        .route("/api/v1/categories", get(category_handler::list_categories))
        .route(
            "/api/v1/categories/:id",
            get(category_handler::get_category),
        )
        .route("/api/v1/categories/search", get(category_handler::search_categories)  )
        .with_state(category_service.clone());

    let category_admin_router = Router::new()
        .route(
            "/api/v1/categories",
            post(category_handler::create_category),
        )
        .route(
            "/api/v1/categories/:id",
            delete(category_handler::delete_category),
        )
        .with_state(category_service)
        .route_layer(middleware::from_fn(jwt_auth::require_admin));

    Router::new()
        .merge(product_public_router)
        .merge(product_admin_router)
        .merge(category_public_router)
        .merge(category_admin_router)
        .route("/metrics", get(metrics::metrics))
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
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
