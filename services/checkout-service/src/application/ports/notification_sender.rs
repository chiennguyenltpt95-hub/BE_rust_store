use async_trait::async_trait;
use domain_core::error::DomainError;

use crate::domain::entities::checkout::Checkout;

#[async_trait]
pub trait NotificationSenderPort: Send + Sync {
    async fn notify_checkout_created(&self, checkout: &Checkout) -> Result<(), DomainError>;
}
