use anyhow::{anyhow, Result};
use dotenvy::from_filename;
use tokio::time::{timeout, Duration};
use tracing::{info, warn};

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
    .map_err(|_| {
        anyhow!(
            "Timed out connecting to database after {}s",
            db_connect_timeout
        )
    })??;

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

    let router =
        presentation::rest::router::build_router(db_pool, cfg.product_service_base_url.clone());

    let addr: std::net::SocketAddr = cfg.http_addr.parse()?;
    info!("Listening on {}", addr);

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
