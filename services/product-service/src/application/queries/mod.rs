use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::domain::entities::product::{Product, ProductStatus};

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ListProductsQuery {
    pub search: Option<String>,
    pub sku: Option<String>,
    pub product_type: Option<String>,
    pub category_id: Option<Uuid>,
    pub category_slug: Option<String>,
    pub category_active: Option<bool>,
    pub target_gender: Option<String>,
    pub min_price_cents: Option<i64>,
    pub max_price_cents: Option<i64>,
    pub in_stock: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProductView {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub sku: Option<String>,
    pub category_id: Option<Uuid>,
    pub category_name: Option<String>,
    pub category_slug: Option<String>,
    pub category_is_active: Option<bool>,
    pub category_target_gender: Option<String>,
    pub product_type: Option<String>,
    pub attributes: Value,
    pub image_urls: Vec<String>,
    pub price_cents: i64,
    pub stock: i32,
    pub status: ProductStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Product> for ProductView {
    fn from(value: Product) -> Self {
        Self {
            id: value.id,
            name: value.name,
            description: value.description,
            sku: value.sku,
            category_id: value.category_id,
            category_name: value.category_name,
            category_slug: value.category_slug,
            category_is_active: value.category_is_active,
            category_target_gender: value.category_target_gender,
            product_type: value.product_type,
            attributes: value.attributes,
            image_urls: value.image_urls,
            price_cents: value.price_cents,
            stock: value.stock,
            status: value.status,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CategoryView {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub parent_id: Option<Uuid>,
    pub display_order: i32,
    pub is_active: bool,
    pub target_gender: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<crate::domain::entities::Category> for CategoryView {
    fn from(value: crate::domain::entities::Category) -> Self {
        Self {
            id: value.id,
            name: value.name,
            slug: value.slug,
            description: value.description,
            image_url: value.image_url,
            parent_id: value.parent_id,
            display_order: value.display_order,
            is_active: value.is_active,
            target_gender: value.target_gender,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
