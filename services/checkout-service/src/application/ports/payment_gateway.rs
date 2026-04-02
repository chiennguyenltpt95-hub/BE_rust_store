use async_trait::async_trait;
use domain_core::error::DomainError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::entities::checkout::PaymentMethod;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRequest {
    pub checkout_id: Uuid,
    pub amount_cents: i64,
    pub currency: String,
    pub description: String,
    pub success_url: String,
    pub cancel_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentResult {
    pub provider: String,
    pub provider_payment_id: String,
    pub checkout_url: String,
    pub status: String,
    pub raw_response: serde_json::Value,
}

#[async_trait]
pub trait PaymentGateway: Send + Sync {
    async fn create_payment(&self, request: &PaymentRequest) -> Result<PaymentResult, DomainError>;
}

pub trait PaymentGatewayFactoryPort: Send + Sync {
    fn get_gateway(&self, method: &PaymentMethod) -> Result<&dyn PaymentGateway, DomainError>;
}
