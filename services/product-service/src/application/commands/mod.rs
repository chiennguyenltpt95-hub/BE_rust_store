use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct CreateProductCommand {
    #[validate(length(min = 2, max = 200))]
    pub name: String,
    pub description: Option<String>,
    #[validate(length(min = 1, max = 64))]
    pub sku: Option<String>,
    pub category_id: Option<Uuid>,
    pub product_type: Option<String>,
    pub attributes: Option<Value>,
    pub image_urls: Option<Vec<String>>,
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
    #[validate(length(min = 1, max = 64))]
    pub sku: Option<String>,
    pub category_id: Option<Uuid>,
    pub product_type: Option<String>,
    pub attributes: Option<Value>,
    pub image_urls: Option<Vec<String>>,
    #[validate(range(min = 0))]
    pub price_cents: i64,
    #[validate(range(min = 0))]
    pub stock: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct CreateCategoryCommand {
    #[validate(length(min = 2, max = 120))]
    pub name: String,
    #[validate(length(min = 2, max = 140))]
    pub slug: Option<String>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub parent_id: Option<Uuid>,
    #[validate(range(min = 0))]
    pub display_order: Option<i32>,
    pub is_active: Option<bool>,
    pub target_gender: Option<String>,
}
