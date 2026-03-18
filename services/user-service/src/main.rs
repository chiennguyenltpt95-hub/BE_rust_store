use anyhow::Result;
use dotenvy::from_filename;
use tracing::info;

mod application;
mod config;
mod domain;
mod infrastructure;
mod presentation;

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env từ thư mục service
    from_filename("services/user-service/.env").ok();
    // Fallback: load từ thư mục hiện tại
    dotenvy::dotenv().ok();

    // Khởi tạo tracing / OpenTelemetry
    infrastructure::telemetry::init_tracing("user-service")?;

    info!("Starting user-service...");

    // Load config
    let cfg = config::AppConfig::from_env()?;

    // Khởi tạo DB pool
    let db_pool = infrastructure::persistence::create_pool(&cfg.database_url).await?;

    // Migrations
    sqlx::migrate!("./migrations").run(&db_pool).await?;

    // Khởi tạo Kafka producer
    let event_publisher = std::sync::Arc::new(infrastructure::messaging::KafkaEventPublisher::new(
        &cfg.kafka_brokers,
        &cfg.kafka_topic,
    )?);

    // Wire dependencies (Composition Root)
    let user_repo = std::sync::Arc::new(
        infrastructure::persistence::user_repository::PgUserRepository::new(db_pool.clone()),
    );
    let user_app_service =
        std::sync::Arc::new(application::services::user_service::UserAppService::new(
            user_repo.clone(),
            event_publisher.clone(),
        ));

    let token_repo = std::sync::Arc::new(
        infrastructure::persistence::token_repository::PgTokenRepository::new(db_pool.clone()),
    );
    let auth_app_service =
        std::sync::Arc::new(application::services::auth_service::AuthAppService::new(
            user_repo.clone(),
            token_repo,
            &cfg.jwt_secret,
        ));

    // Khởi router HTTP
    let router = presentation::rest::router::build_router(user_app_service, auth_app_service);

    let addr: std::net::SocketAddr = cfg.http_addr.parse()?;
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}
