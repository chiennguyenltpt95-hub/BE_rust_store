use anyhow::Result;
use dotenvy::from_filename;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::info;

mod application;
mod config;
mod infrastructure;
mod presentation;

#[tokio::main]
async fn main() -> Result<()> {
    from_filename("services/notification-service/.env").ok();
    dotenvy::dotenv().ok();

    infrastructure::telemetry::init_tracing("notification-service")?;
    info!("Starting notification-service...");

    let cfg = config::AppConfig::from_env()?;

    let db_pool = infrastructure::persistence::create_pool(&cfg.database_url).await?;
    let mut migrator = sqlx::migrate!("./migrations");
    migrator.set_ignore_missing(true);
    migrator.run(&db_pool).await?;

    let channel_factory = Arc::new(application::channels::ChannelFactory::new(
        cfg.telegram_bot_token.clone(),
    ));

    let worker_pool = db_pool.clone();
    let worker_factory = Arc::clone(&channel_factory);
    let poll_interval = cfg.worker_poll_interval_secs;
    let worker_batch_size = cfg.worker_batch_size;

    let kafka_brokers = cfg.kafka_brokers.clone();
    let kafka_topic = cfg.kafka_topic.clone();
    let kafka_group_id = cfg.kafka_group_id.clone();
    let kafka_pool = db_pool.clone();
    let default_max_attempts = cfg.default_max_attempts;
    let default_telegram_recipient = cfg.telegram_channel_id.clone();
    tokio::spawn(async move {
        if let Err(err) = presentation::event_listener::start_event_listener(
            &kafka_brokers,
            &kafka_topic,
            &kafka_group_id,
            kafka_pool,
            default_max_attempts,
            default_telegram_recipient,
        )
        .await
        {
            tracing::error!("Notification Kafka listener error: {}", err);
        }
    });

    tokio::spawn(async move {
        loop {
            if let Err(err) = application::worker::process_once(
                &worker_pool,
                Arc::clone(&worker_factory),
                worker_batch_size,
            )
            .await
            {
                tracing::error!("notification worker iteration failed: {}", err);
            }

            sleep(Duration::from_secs(poll_interval)).await;
        }
    });

    let router = presentation::rest::router::build_router(db_pool, cfg.default_max_attempts);

    let addr: std::net::SocketAddr = cfg.http_addr.parse()?;
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}
