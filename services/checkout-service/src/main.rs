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
    from_filename("services/checkout-service/.env").ok();
    dotenvy::dotenv().ok();

    infrastructure::telemetry::init_tracing("checkout-service")?;
    info!("Starting checkout-service...");

    let cfg = config::AppConfig::from_env()?;

    let db_pool = infrastructure::persistence::create_pool(&cfg.database_url).await?;
    let mut migrator = sqlx::migrate!("./migrations");
    migrator.set_ignore_missing(true);
    migrator.run(&db_pool).await?;

    info!("Database connected and migrations applied and migrations applied");

    let checkout_repo = Arc::new(infrastructure::persistence::PgCheckoutRepository::new(
        db_pool,
    ));

    let payment_factory = Arc::new(infrastructure::payment::PaymentGatewayFactory::new(
        cfg.clone(),
    ));
    let cart_reader = Arc::new(infrastructure::http::CartServiceClient::new(
        cfg.cart_service_base_url.clone(),
    ));
    let notification_sender = Arc::new(infrastructure::messaging::KafkaNotificationPublisher::new(
        &cfg.kafka_brokers,
        &cfg.kafka_topic,
        cfg.telegram_chat_id.clone(),
    )?);

    let checkout_service = Arc::new(application::services::CheckoutAppService::new(
        checkout_repo,
        payment_factory,
        cart_reader,
        notification_sender,
        cfg.paypal_webhook_secret.clone(),
        cfg.stripe_webhook_secret.clone(),
        cfg.oxapay_webhook_secret.clone(),
        cfg.checkout_success_url.clone(),
        cfg.checkout_cancel_url.clone(),
    ));

    let router = presentation::rest::router::build_router(checkout_service);

    let addr: std::net::SocketAddr = cfg.http_addr.parse()?;
    info!(service = %cfg.service_name, "Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}
