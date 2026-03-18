use anyhow::Result;
use dotenvy::from_filename;
use std::sync::Arc;
use tracing::info;

mod application;
mod config;
mod domain;
mod infrastructure;
mod presentation;

use domain::models::MailAddress;
use domain::ports::MailTransport;

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env
    from_filename("services/mail-service/.env").ok();
    dotenvy::dotenv().ok();

    // Tracing
    infrastructure::telemetry::init_tracing("mail-service")?;
    info!("Starting mail-service...");

    // Config
    let cfg = config::AppConfig::from_env()?;

    // ═══════════════════════════════════════════════════════════════
    // COMPOSITION ROOT (Factory Pattern)
    // Chọn transport dựa trên config — thay đổi provider ở đây,
    // KHÔNG cần sửa bất kỳ code nào khác.
    // ═══════════════════════════════════════════════════════════════
    let transport: Arc<dyn MailTransport> = match cfg.mail_transport.as_str() {
        "console" => {
            info!("Using ConsoleTransport (dev mode)");
            Arc::new(infrastructure::transports::ConsoleTransport)
        }
        _ => {
            info!("Using SmtpTransport: {}:{}", cfg.smtp_host, cfg.smtp_port);
            Arc::new(infrastructure::transports::SmtpTransport::new(
                &cfg.smtp_host,
                cfg.smtp_port,
                &cfg.smtp_username,
                &cfg.smtp_password,
            )?)
        }
    };

    // Template engine
    let template_engine = Arc::new(infrastructure::templates::TeraTemplateEngine::new()?);

    // Default "from" address
    let default_from = MailAddress::with_name(&cfg.mail_from_email, &cfg.mail_from_name);

    // Application service
    let mail_svc = Arc::new(application::services::MailAppService::new(
        transport,
        template_engine,
        default_from,
    ));

    // ── Kafka Event Listener (background task) ─────────────────
    let kafka_brokers = cfg.kafka_brokers.clone();
    let kafka_topic = cfg.kafka_topic.clone();
    let kafka_group = cfg.kafka_group_id.clone();
    let mail_svc_clone = mail_svc.clone();
    tokio::spawn(async move {
        if let Err(e) = presentation::event_listener::start_event_listener(
            &kafka_brokers,
            &kafka_topic,
            &kafka_group,
            mail_svc_clone,
        )
        .await
        {
            tracing::error!("Kafka event listener error: {}", e);
        }
    });

    // ── HTTP Server ──────────────────────────────────────────────
    let router = presentation::rest::router::build_router(mail_svc);

    let addr: std::net::SocketAddr = cfg.http_addr.parse()?;
    info!("Listening on {}", addr);
    info!("Swagger UI: http://{}/swagger-ui", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}
