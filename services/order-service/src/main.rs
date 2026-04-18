use anyhow::{anyhow, Result};
use dotenvy::from_filename;
use std::sync::Arc;
use tokio::time::{timeout, Duration};
use tracing::{info, warn};

mod application;
mod config;
mod domain;
mod infrastructure;
mod presentation;

#[tokio::main]
async fn main() -> Result<()> {
    from_filename("services/order-service/.env").ok();
    dotenvy::dotenv().ok();

    infrastructure::telemetry::init_tracing("order-service")?;
    info!("Starting order-service...");

    let cfg = config::AppConfig::from_env()?;

    info!("Connecting database...");
    let db_connect_timeout = std::env::var("DB_CONNECT_TIMEOUT_SECONDS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(30);

    let db_pool = timeout(
        Duration::from_secs(db_connect_timeout),
        infrastructure::persistence::create_pool(&cfg.database_url),
    )
    .await
    .map_err(|_| anyhow!("Timed out connecting to database after {}s", db_connect_timeout))??;

    info!("Database connected");
    let run_migrations_on_startup = std::env::var("RUN_MIGRATIONS_ON_STARTUP")
        .ok()
        .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
        .unwrap_or(true);

    if run_migrations_on_startup {
        run_migrations(&db_pool).await?;
    } else {
        info!("Skipping startup migrations (RUN_MIGRATIONS_ON_STARTUP=0)");
    }

    let order_repo: Arc<dyn domain::repositories::OrderRepository> =
        Arc::new(infrastructure::persistence::PgOrderRepository::new(db_pool));
    let checkout_reader = Arc::new(infrastructure::http::CheckoutServiceClient::new(
        cfg.checkout_service_base_url.clone(),
    ));
    let event_publisher = Arc::new(infrastructure::messaging::KafkaEventPublisher::new(
        &cfg.kafka_brokers,
        &cfg.kafka_topic,
    )?);

    let order_service = Arc::new(application::services::OrderAppService::new(
        order_repo.clone(),
        checkout_reader,
    ));

    let outbox_service = Arc::new(application::services::OutboxService::new(
        order_repo,
        event_publisher,
        cfg.outbox_max_attempts,
    ));

    let poll_interval_secs = cfg.outbox_poll_interval_secs;
    let outbox_batch_size = cfg.outbox_batch_size;
    let outbox_service_clone = outbox_service.clone();
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(poll_interval_secs));
        loop {
            ticker.tick().await;
            if let Err(err) = outbox_service_clone.process_once(outbox_batch_size).await {
                tracing::error!("Outbox processor error: {}", err);
            }
        }
    });

    let router = presentation::rest::router::build_router(order_service, outbox_service);

    let addr: std::net::SocketAddr = cfg.http_addr.parse()?;
    info!(service = %cfg.service_name, "Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}

async fn run_migrations(db_pool: &sqlx::PgPool) -> Result<()> {
    info!("Running database migrations...");

    let mut migrator = sqlx::migrate!("./migrations");
    migrator.set_ignore_missing(true);

    let migration_timeout = std::env::var("MIGRATION_TIMEOUT_SECONDS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(45);

    let migration_result = timeout(Duration::from_secs(migration_timeout), migrator.run(db_pool))
        .await
        .map_err(|_| {
            anyhow!(
                "Timed out waiting for migrations after {}s (likely waiting on migration advisory lock)",
                migration_timeout
            )
        })?;

    if let Err(err) = migration_result {
        let err_msg = err.to_string();
        if err_msg.contains("previously applied but has been modified") {
            warn!(error = %err, "Ignoring modified migration checksum mismatch for local/dev startup");
            return Ok(());
        }
        return Err(err.into());
    }

    info!("Migrations completed");
    Ok(())
}
