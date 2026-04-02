use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct CreateProductCommand {
    #[validate(length(min = 2, max = 200))]
    pub name: String,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
    #[validate(range(min = 0))]
    pub price_cents: i64,
    #[validate(range(min = 0))]
    pub stock: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct UpdateProductCommand {
    #[validate(length(min = 2, max = 200))]
    pub name: String,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
    #[validate(range(min = 0))]
    pub price_cents: i64,
    #[validate(range(min = 0))]
    pub stock: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct CreateCategoryCommand {
    #[validate(length(min = 2, max = 120))]
    pub name: String,
    #[validate(length(min = 2, max = 120))]
    pub slug: Option<String>,
}
