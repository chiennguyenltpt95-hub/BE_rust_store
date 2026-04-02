use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateOrderItemCommand {
    pub product_id: Uuid,
    #[validate(length(min = 1, max = 255))]
    pub product_name: String,
    #[validate(range(min = 1))]
    pub quantity: i32,
    #[validate(range(min = 0))]
    pub unit_price_cents: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateOrderCommand {
    pub user_id: Uuid,
    pub checkout_id: Uuid,
    pub cart_id: Uuid,
    #[validate(email)]
    pub customer_email: String,
    #[validate(length(min = 1, max = 255))]
    pub customer_name: String,
    #[validate(length(min = 3, max = 16))]
    pub currency: String,
    #[validate(length(min = 8, max = 128))]
    pub idempotency_key: Option<String>,
    #[validate(length(min = 1))]
    pub items: Vec<CreateOrderItemCommand>,
}
