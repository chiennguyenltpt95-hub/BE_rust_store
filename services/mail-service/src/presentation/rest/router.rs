use axum::{routing::get, Router};
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::CorsLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use super::{mail_handler, metrics};
use crate::application::services::MailAppService;

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "Mail", description = "Mail sending endpoints")
    )
)]
struct ApiDoc;

pub fn build_router(mail_svc: Arc<MailAppService>) -> Router {
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(utoipa_axum::routes!(mail_handler::send_raw_mail))
        .routes(utoipa_axum::routes!(mail_handler::send_templated_mail))
        .with_state(mail_svc)
        .split_for_parts();

    router
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api))
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
