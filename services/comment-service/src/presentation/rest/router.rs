use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::CorsLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use super::{comment_handler, jwt_auth, metrics};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub product_service_base_url: Arc<str>,
}

pub fn build_router(pool: PgPool, product_service_base_url: String) -> Router {
    let state = AppState {
        pool,
        product_service_base_url: Arc::from(product_service_base_url.into_boxed_str()),
    };

    let public_router = Router::new()
        .route("/api/v1/comments/:id", get(comment_handler::get_comment))
        .route(
            "/api/v1/products/:product_id/comments",
            get(comment_handler::list_product_comments),
        )
        .with_state(state.clone());

    let protected_router = Router::new()
        .route("/api/v1/comments", post(comment_handler::create_comment))
        .route("/api/v1/comments/:id", put(comment_handler::update_comment))
        .route(
            "/api/v1/comments/:id",
            delete(comment_handler::delete_comment),
        )
        .with_state(state.clone())
        .route_layer(middleware::from_fn(jwt_auth::require_jwt));

    Router::new()
        .merge(public_router)
        .merge(protected_router)
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
