use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use validator::Validate;

use crate::domain::entities::product::ProductStatus;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProductForm {
    pub name: String,
    pub sku: String,
    pub category_id: String,
    pub supplier_id: String,
    pub supplier_url: String,
    pub cost_price: String,
    pub price: String,
    pub sale_price: String,
    pub description: String,
    pub images: Vec<String>,
    pub sizes: Vec<String>,
    pub colors: Vec<String>,
    pub stock: String,
    pub weight: String,
    pub status: ProductStatus,
    pub tags: String,
}

impl TryFrom<CreateProductForm> for CreateProductCommand {
    type Error = String;

    fn try_from(value: CreateProductForm) -> Result<Self, Self::Error> {
        let price_cents = parse_required_non_negative_i64(&value.price, "price")?;
        let stock = parse_required_non_negative_i32(&value.stock, "stock")?;

        Ok(Self {
            name: value.name.trim().to_string(),
            description: empty_to_none(value.description),
            sku: empty_to_none(value.sku),
            category_id: parse_optional_uuid(&value.category_id, "categoryId")?,
            supplier_id: parse_optional_uuid(&value.supplier_id, "supplierId")?,
            supplier_url: empty_to_none(value.supplier_url),
            product_type: None,
            attributes: None,
            image_urls: sanitize_list(value.images),
            price_cents,
            cost_price_cents: parse_optional_non_negative_i64(&value.cost_price, "costPrice")?,
            sale_price_cents: parse_optional_non_negative_i64(&value.sale_price, "salePrice")?,
            sizes: sanitize_list(value.sizes),
            colors: sanitize_list(value.colors),
            stock,
            weight_grams: parse_optional_non_negative_i32(&value.weight, "weight")?,
            status: value.status,
            tags: split_tags(&value.tags),
        })
    }
}

fn empty_to_none(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn sanitize_list(items: Vec<String>) -> Vec<String> {
    items
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn split_tags(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn parse_required_non_negative_i64(value: &str, field: &str) -> Result<i64, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!("{} is required", field));
    }

    let parsed = trimmed
        .parse::<i64>()
        .map_err(|_| format!("{} must be a valid integer", field))?;
    if parsed < 0 {
        return Err(format!("{} cannot be negative", field));
    }

    Ok(parsed)
}

fn parse_optional_non_negative_i64(value: &str, field: &str) -> Result<Option<i64>, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let parsed = trimmed
        .parse::<i64>()
        .map_err(|_| format!("{} must be a valid integer", field))?;
    if parsed < 0 {
        return Err(format!("{} cannot be negative", field));
    }

    Ok(Some(parsed))
}

fn parse_required_non_negative_i32(value: &str, field: &str) -> Result<i32, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!("{} is required", field));
    }

    let parsed = trimmed
        .parse::<i32>()
        .map_err(|_| format!("{} must be a valid integer", field))?;
    if parsed < 0 {
        return Err(format!("{} cannot be negative", field));
    }

    Ok(parsed)
}

fn parse_optional_non_negative_i32(value: &str, field: &str) -> Result<Option<i32>, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let parsed = trimmed
        .parse::<i32>()
        .map_err(|_| format!("{} must be a valid integer", field))?;
    if parsed < 0 {
        return Err(format!("{} cannot be negative", field));
    }

    Ok(Some(parsed))
}

fn parse_optional_uuid(value: &str, field: &str) -> Result<Option<Uuid>, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let parsed = Uuid::parse_str(trimmed).map_err(|_| format!("{} must be a valid UUID", field))?;
    Ok(Some(parsed))
}

fn default_product_status() -> ProductStatus {
    ProductStatus::Active
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateProductCommand {
    #[validate(length(min = 2, max = 200))]
    pub name: String,
    pub description: Option<String>,
    #[validate(length(min = 1, max = 64))]
    pub sku: Option<String>,
    pub category_id: Option<Uuid>,
    pub supplier_id: Option<Uuid>,
    pub supplier_url: Option<String>,
    pub product_type: Option<String>,
    pub attributes: Option<Value>,
    #[serde(default)]
    pub image_urls: Vec<String>,
    #[validate(range(min = 0))]
    pub price_cents: i64,
    #[validate(range(min = 0))]
    pub cost_price_cents: Option<i64>,
    #[validate(range(min = 0))]
    pub sale_price_cents: Option<i64>,
    #[serde(default)]
    pub sizes: Vec<String>,
    #[serde(default)]
    pub colors: Vec<String>,
    #[validate(range(min = 0))]
    pub stock: i32,
    #[validate(range(min = 0))]
    pub weight_grams: Option<i32>,
    #[serde(default = "default_product_status")]
    pub status: ProductStatus,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProductCommand {
    #[validate(length(min = 2, max = 200))]
    pub name: String,
    pub description: Option<String>,
    #[validate(length(min = 1, max = 64))]
    pub sku: Option<String>,
    pub category_id: Option<Uuid>,
    pub supplier_id: Option<Uuid>,
    pub supplier_url: Option<String>,
    pub product_type: Option<String>,
    pub attributes: Option<Value>,
    #[serde(default)]
    pub image_urls: Vec<String>,
    #[validate(range(min = 0))]
    pub price_cents: i64,
    #[validate(range(min = 0))]
    pub cost_price_cents: Option<i64>,
    #[validate(range(min = 0))]
    pub sale_price_cents: Option<i64>,
    #[serde(default)]
    pub sizes: Vec<String>,
    #[serde(default)]
    pub colors: Vec<String>,
    #[validate(range(min = 0))]
    pub stock: i32,
    #[validate(range(min = 0))]
    pub weight_grams: Option<i32>,
    #[serde(default = "default_product_status")]
    pub status: ProductStatus,
    #[serde(default)]
    pub tags: Vec<String>,
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
