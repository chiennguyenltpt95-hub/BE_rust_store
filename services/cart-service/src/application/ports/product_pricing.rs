use async_trait::async_trait;
use domain_core::error::DomainError;
use uuid::Uuid;

#[async_trait]
pub trait ProductPricingPort: Send + Sync {
    async fn get_product_price_cents(&self, product_id: Uuid) -> Result<i64, DomainError>;
}
