use axum::Router;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use crate::application::services::MailAppService;
use super::mail_handler;

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
        .layer(CorsLayer::permissive())
}
