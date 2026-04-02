use async_trait::async_trait;
use domain_core::error::DomainError;
use uuid::Uuid;

#[async_trait]
pub trait CartReaderPort: Send + Sync {
    async fn get_cart_total_cents(&self, cart_id: Uuid) -> Result<i64, DomainError>;
}
