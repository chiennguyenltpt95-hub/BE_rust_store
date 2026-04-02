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
    from_filename("services/order-service/.env").ok();
    dotenvy::dotenv().ok();

    infrastructure::telemetry::init_tracing("order-service")?;
    info!("Starting order-service...");

    let cfg = config::AppConfig::from_env()?;

    let db_pool = infrastructure::persistence::create_pool(&cfg.database_url).await?;
    let mut migrator = sqlx::migrate!("./migrations");
    migrator.set_ignore_missing(true);
    migrator.run(&db_pool).await?;

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
