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
    from_filename("services/product-service/.env").ok();
    dotenvy::dotenv().ok();

    infrastructure::telemetry::init_tracing("product-service")?;
    info!("Starting product-service...");

    let cfg = config::AppConfig::from_env()?;
    std::env::set_var("JWT_SECRET", &cfg.jwt_secret);

    let db_pool = infrastructure::persistence::create_pool(&cfg.database_url).await?;
    let mut migrator = sqlx::migrate!("./migrations");
    migrator.set_ignore_missing(true);
    migrator.run(&db_pool).await?;

    let product_repo = Arc::new(infrastructure::persistence::PgProductRepository::new(
        db_pool,
    ));
    let product_service = Arc::new(application::services::ProductAppService::new(
        product_repo.clone(),
    ));
    let category_service = Arc::new(application::services::CategoryAppService::new(product_repo));

    let router = presentation::rest::router::build_router(product_service, category_service);

    let addr: std::net::SocketAddr = cfg.http_addr.parse()?;
    info!(service = %cfg.service_name, "Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}
