use chrono::{DateTime, Utc};
use domain_core::error::DomainError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProductStatus {
    Draft,
    Active,
    Inactive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub sku: Option<String>,
    pub category_id: Option<Uuid>,
    pub supplier_id: Option<Uuid>,
    pub supplier_url: Option<String>,
    pub category_name: Option<String>,
    pub category_slug: Option<String>,
    pub category_is_active: Option<bool>,
    pub category_target_gender: Option<String>,
    pub product_type: Option<String>,
    pub attributes: Value,
    pub image_urls: Vec<String>,
    pub price_cents: i64,
    pub cost_price_cents: Option<i64>,
    pub sale_price_cents: Option<i64>,
    pub sizes: Vec<String>,
    pub colors: Vec<String>,
    pub stock: i32,
    pub weight_grams: Option<i32>,
    pub tags: Vec<String>,
    pub status: ProductStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Product {
    pub fn create(
        id: Uuid,
        name: String,
        description: Option<String>,
        sku: Option<String>,
        category_id: Option<Uuid>,
        supplier_id: Option<Uuid>,
        supplier_url: Option<String>,
        product_type: Option<String>,
        attributes: Value,
        image_urls: Vec<String>,
        price_cents: i64,
        cost_price_cents: Option<i64>,
        sale_price_cents: Option<i64>,
        sizes: Vec<String>,
        colors: Vec<String>,
        stock: i32,
        weight_grams: Option<i32>,
        status: ProductStatus,
        tags: Vec<String>,
    ) -> Result<Self, DomainError> {
        if name.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Product name cannot be empty".into(),
            ));
        }
        if price_cents < 0 {
            return Err(DomainError::ValidationError(
                "Product price cannot be negative".into(),
            ));
        }
        if let Some(cost_price) = cost_price_cents {
            if cost_price < 0 {
                return Err(DomainError::ValidationError(
                    "Product cost price cannot be negative".into(),
                ));
            }
            if cost_price > price_cents {
                return Err(DomainError::ValidationError(
                    "Product cost price cannot exceed selling price".into(),
                ));
            }
        }
        if let Some(sale_price) = sale_price_cents {
            if sale_price < 0 {
                return Err(DomainError::ValidationError(
                    "Product sale price cannot be negative".into(),
                ));
            }
            if sale_price > price_cents {
                return Err(DomainError::ValidationError(
                    "Product sale price cannot exceed selling price".into(),
                ));
            }
        }
        if stock < 0 {
            return Err(DomainError::ValidationError(
                "Product stock cannot be negative".into(),
            ));
        }
        if let Some(weight) = weight_grams {
            if weight < 0 {
                return Err(DomainError::ValidationError(
                    "Product weight cannot be negative".into(),
                ));
            }
        }

        if let Some(sku_value) = &sku {
            if sku_value.trim().is_empty() {
                return Err(DomainError::ValidationError(
                    "Product sku cannot be empty".into(),
                ));
            }
            if sku_value.len() > 64 {
                return Err(DomainError::ValidationError(
                    "Product sku must be at most 64 characters".into(),
                ));
            }
        }

        if image_urls.len() > 20 {
            return Err(DomainError::ValidationError(
                "Product image_urls supports up to 20 images".into(),
            ));
        }

        if sizes.len() > 30 {
            return Err(DomainError::ValidationError(
                "Product sizes supports up to 30 entries".into(),
            ));
        }

        if colors.len() > 30 {
            return Err(DomainError::ValidationError(
                "Product colors supports up to 30 entries".into(),
            ));
        }

        if tags.len() > 50 {
            return Err(DomainError::ValidationError(
                "Product tags supports up to 50 entries".into(),
            ));
        }

        if let Some(kind) = &product_type {
            if kind.trim().is_empty() {
                return Err(DomainError::ValidationError(
                    "Product type cannot be empty".into(),
                ));
            }
        }

        if !attributes.is_object() {
            return Err(DomainError::ValidationError(
                "Product attributes must be a JSON object".into(),
            ));
        }

        let now = Utc::now();

        Ok(Self {
            id,
            name,
            description,
            sku,
            category_id,
            supplier_id,
            supplier_url,
            category_name: None,
            category_slug: None,
            category_is_active: None,
            category_target_gender: None,
            product_type,
            attributes,
            image_urls,
            price_cents,
            cost_price_cents,
            sale_price_cents,
            sizes,
            colors,
            stock,
            weight_grams,
            tags,
            status,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn update(
        &mut self,
        name: String,
        description: Option<String>,
        sku: Option<String>,
        category_id: Option<Uuid>,
        supplier_id: Option<Uuid>,
        supplier_url: Option<String>,
        product_type: Option<String>,
        attributes: Value,
        image_urls: Vec<String>,
        price_cents: i64,
        cost_price_cents: Option<i64>,
        sale_price_cents: Option<i64>,
        sizes: Vec<String>,
        colors: Vec<String>,
        stock: i32,
        weight_grams: Option<i32>,
        status: ProductStatus,
        tags: Vec<String>,
    ) -> Result<(), DomainError> {
        if name.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Product name cannot be empty".into(),
            ));
        }
        if price_cents < 0 {
            return Err(DomainError::ValidationError(
                "Product price cannot be negative".into(),
            ));
        }
        if let Some(cost_price) = cost_price_cents {
            if cost_price < 0 {
                return Err(DomainError::ValidationError(
                    "Product cost price cannot be negative".into(),
                ));
            }
            if cost_price > price_cents {
                return Err(DomainError::ValidationError(
                    "Product cost price cannot exceed selling price".into(),
                ));
            }
        }
        if let Some(sale_price) = sale_price_cents {
            if sale_price < 0 {
                return Err(DomainError::ValidationError(
                    "Product sale price cannot be negative".into(),
                ));
            }
            if sale_price > price_cents {
                return Err(DomainError::ValidationError(
                    "Product sale price cannot exceed selling price".into(),
                ));
            }
        }
        if stock < 0 {
            return Err(DomainError::ValidationError(
                "Product stock cannot be negative".into(),
            ));
        }
        if let Some(weight) = weight_grams {
            if weight < 0 {
                return Err(DomainError::ValidationError(
                    "Product weight cannot be negative".into(),
                ));
            }
        }

        if let Some(sku_value) = &sku {
            if sku_value.trim().is_empty() {
                return Err(DomainError::ValidationError(
                    "Product sku cannot be empty".into(),
                ));
            }
            if sku_value.len() > 64 {
                return Err(DomainError::ValidationError(
                    "Product sku must be at most 64 characters".into(),
                ));
            }
        }

        if image_urls.len() > 20 {
            return Err(DomainError::ValidationError(
                "Product image_urls supports up to 20 images".into(),
            ));
        }

        if sizes.len() > 30 {
            return Err(DomainError::ValidationError(
                "Product sizes supports up to 30 entries".into(),
            ));
        }

        if colors.len() > 30 {
            return Err(DomainError::ValidationError(
                "Product colors supports up to 30 entries".into(),
            ));
        }

        if tags.len() > 50 {
            return Err(DomainError::ValidationError(
                "Product tags supports up to 50 entries".into(),
            ));
        }

        if let Some(kind) = &product_type {
            if kind.trim().is_empty() {
                return Err(DomainError::ValidationError(
                    "Product type cannot be empty".into(),
                ));
            }
        }

        if !attributes.is_object() {
            return Err(DomainError::ValidationError(
                "Product attributes must be a JSON object".into(),
            ));
        }

        self.name = name;
        self.description = description;
        self.sku = sku;
        self.category_id = category_id;
        self.supplier_id = supplier_id;
        self.supplier_url = supplier_url;
        self.product_type = product_type;
        self.attributes = attributes;
        self.image_urls = image_urls;
        self.price_cents = price_cents;
        self.cost_price_cents = cost_price_cents;
        self.sale_price_cents = sale_price_cents;
        self.sizes = sizes;
        self.colors = colors;
        self.stock = stock;
        self.weight_grams = weight_grams;
        self.status = status;
        self.tags = tags;
        self.updated_at = Utc::now();

        Ok(())
    }

    pub fn deactivate(&mut self) {
        self.status = ProductStatus::Inactive;
        self.updated_at = Utc::now();
    }
}
