pub mod pg_cart_repository;

use anyhow::Result;
use sqlx::PgPool;

pub use pg_cart_repository::PgCartRepository;

pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;
    Ok(pool)
}
