use async_trait::async_trait;
use domain_core::error::DomainError;
use uuid::Uuid;

use crate::domain::entities::{Checkout, PaymentTransaction};

#[async_trait]
pub trait CheckoutRepository: Send + Sync {
    async fn create_checkout(&self, checkout: &Checkout) -> Result<(), DomainError>;
    async fn find_checkout_by_id(&self, checkout_id: Uuid)
        -> Result<Option<Checkout>, DomainError>;
    async fn find_checkout_by_idempotency_key(
        &self,
        idempotency_key: &str,
    ) -> Result<Option<Checkout>, DomainError>;
    async fn update_checkout(&self, checkout: &Checkout) -> Result<(), DomainError>;
    async fn transaction_exists(
        &self,
        provider: &str,
        provider_payment_id: &str,
    ) -> Result<bool, DomainError>;
    async fn create_transaction(&self, transaction: &PaymentTransaction)
        -> Result<(), DomainError>;
    async fn list_transactions_by_checkout(
        &self,
        checkout_id: Uuid,
    ) -> Result<Vec<PaymentTransaction>, DomainError>;
}
