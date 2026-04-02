use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct CreateCheckoutCommand {
    pub user_id: Uuid,
    pub cart_id: Uuid,
    #[validate(length(min = 3, max = 16))]
    pub currency: String,
    pub payment_method: String,
    #[validate(length(min = 8, max = 128))]
    pub idempotency_key: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct MarkPaidCommand {
    pub provider_payment_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct MarkFailedCommand {
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct ProviderWebhookCommand {
    pub checkout_id: Uuid,
    #[validate(length(min = 1))]
    pub provider_payment_id: String,
    #[validate(length(min = 1))]
    pub status: String,
    pub reason: Option<String>,
}
