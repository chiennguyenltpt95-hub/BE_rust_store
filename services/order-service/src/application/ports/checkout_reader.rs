use async_trait::async_trait;
use domain_core::error::DomainError;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CheckoutSnapshot {
    pub amount_cents: i64,
    pub currency: String,
    pub status: String,
}

#[async_trait]
pub trait CheckoutReaderPort: Send + Sync {
    async fn get_checkout(&self, checkout_id: Uuid) -> Result<CheckoutSnapshot, DomainError>;
}
