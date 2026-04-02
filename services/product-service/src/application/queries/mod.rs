use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::entities::product::{Product, ProductStatus};

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ListProductsQuery {
    pub search: Option<String>,
    pub category_id: Option<Uuid>,
    pub min_price_cents: Option<i64>,
    pub max_price_cents: Option<i64>,
    pub in_stock: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProductView {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
    pub category_name: Option<String>,
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
            category_id: value.category_id,
            category_name: value.category_name,
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<crate::domain::entities::Category> for CategoryView {
    fn from(value: crate::domain::entities::Category) -> Self {
        Self {
            id: value.id,
            name: value.name,
            slug: value.slug,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
