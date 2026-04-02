use anyhow::Result;
use dotenvy::from_filename;
use tracing::info;

mod config;
mod infrastructure;
mod presentation;

#[tokio::main]
async fn main() -> Result<()> {
    from_filename("services/comment-service/.env").ok();
    dotenvy::dotenv().ok();

    infrastructure::telemetry::init_tracing("comment-service")?;
    info!("Starting comment-service...");

    let cfg = config::AppConfig::from_env()?;
    std::env::set_var("JWT_SECRET", &cfg.jwt_secret);

    let db_pool = infrastructure::persistence::create_pool(&cfg.database_url).await?;
    let mut migrator = sqlx::migrate!("./migrations");
    migrator.set_ignore_missing(true);
    migrator.run(&db_pool).await?;

    let router =
        presentation::rest::router::build_router(db_pool, cfg.product_service_base_url.clone());

    let addr: std::net::SocketAddr = cfg.http_addr.parse()?;
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}
