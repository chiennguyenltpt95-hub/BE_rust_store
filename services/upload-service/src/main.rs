use anyhow::Result;
use dotenvy::from_filename;
use std::sync::Arc;
use tracing::info;

mod application;
mod config;
mod infrastructure;
mod presentation;

#[tokio::main]
async fn main() -> Result<()> {
    from_filename("services/upload-service/.env").ok();
    dotenvy::dotenv().ok();

    infrastructure::telemetry::init_tracing("upload-service")?;
    info!("Starting upload-service...");

    let cfg = config::AppConfig::from_env()?;
    std::env::set_var("JWT_SECRET", &cfg.jwt_secret);

    let upload_service = Arc::new(
        application::services::UploadAppService::new(
            cfg.aws_region.clone(),
            cfg.s3_bucket.clone(),
            cfg.s3_key_prefix.clone(),
            cfg.s3_presign_expires_seconds,
            cfg.s3_endpoint_url.clone(),
        )
        .await?,
    );

    let router = presentation::rest::router::build_router(upload_service);

    let addr: std::net::SocketAddr = cfg.http_addr.parse()?;
    info!(service = %cfg.service_name, "Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}
