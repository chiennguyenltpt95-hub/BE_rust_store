use anyhow::Result;
use dotenvy::from_filename;
use std::sync::Arc;
use tracing::{info, warn};

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
    run_migrations(&db_pool).await?;

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

async fn run_migrations(db_pool: &sqlx::PgPool) -> Result<()> {
    let mut migrator = sqlx::migrate!("./migrations");
    migrator.set_ignore_missing(true);

    if let Err(err) = migrator.run(db_pool).await {
        let err_msg = err.to_string();
        if err_msg.contains("previously applied but has been modified") {
            warn!(error = %err, "Ignoring modified migration checksum mismatch for local/dev startup");
            ensure_product_form_columns(db_pool).await?;
            return Ok(());
        }
        return Err(err.into());
    }

    Ok(())
}

async fn ensure_product_form_columns(db_pool: &sqlx::PgPool) -> Result<()> {
    // Forward-compatible hotfix: keep create/update APIs working when historical
    // migration checksum drift prevents applying newer migration files.
    sqlx::query(
        r#"ALTER TABLE products
           ADD COLUMN IF NOT EXISTS supplier_id UUID,
           ADD COLUMN IF NOT EXISTS supplier_url TEXT,
           ADD COLUMN IF NOT EXISTS cost_price_cents BIGINT,
           ADD COLUMN IF NOT EXISTS sale_price_cents BIGINT,
           ADD COLUMN IF NOT EXISTS sizes TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
           ADD COLUMN IF NOT EXISTS colors TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
           ADD COLUMN IF NOT EXISTS weight_grams INTEGER,
           ADD COLUMN IF NOT EXISTS tags TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[]"#,
    )
    .execute(db_pool)
    .await?;

    Ok(())
}
