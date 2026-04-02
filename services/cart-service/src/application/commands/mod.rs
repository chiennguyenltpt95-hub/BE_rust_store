use serde::{Deserialize, Serialize};
use validator::Validate;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct CreateCartCommand {
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct AddItemCommand {
    pub product_id: Uuid,
    #[validate(range(min = 1))]
    pub quantity: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct UpdateItemQuantityCommand {
    #[validate(range(min = 1))]
    pub quantity: i32,
}

