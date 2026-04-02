use anyhow::Result;
use dotenvy::from_filename;
use std::sync::Arc;
use tracing::info;

mod application;
mod config;
mod domain;
mod infrastructure;
mod presentation;

#[tokio::main]
async fn main() -> Result<()> {
    from_filename("services/cart-service/.env").ok();
    dotenvy::dotenv().ok();

    infrastructure::telemetry::init_tracing("cart-service")?;
    info!("Starting cart-service...");

    let cfg = config::AppConfig::from_env()?;

    let db_pool = infrastructure::persistence::create_pool(&cfg.database_url).await?;
    let mut migrator = sqlx::migrate!("./migrations");
    migrator.set_ignore_missing(true);
    migrator.run(&db_pool).await?;

    let cart_repo = Arc::new(infrastructure::persistence::PgCartRepository::new(db_pool));
    let product_pricing = Arc::new(infrastructure::http::ProductServiceClient::new(
        cfg.product_service_base_url.clone(),
    ));
    let cart_service = Arc::new(application::services::CartAppService::new(
        cart_repo,
        product_pricing,
    ));

    let router = presentation::rest::router::build_router(cart_service);

    let addr: std::net::SocketAddr = cfg.http_addr.parse()?;
    info!(service = %cfg.service_name, "Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}
