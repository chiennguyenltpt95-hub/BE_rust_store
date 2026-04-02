use chrono::{DateTime, Utc};
use domain_core::error::DomainError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
    pub category_id: Option<Uuid>,
    pub category_name: Option<String>,
    pub price_cents: i64,
    pub stock: i32,
    pub status: ProductStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Product {
    pub fn create(
        id: Uuid,
        name: String,
        description: Option<String>,
        category_id: Option<Uuid>,
        price_cents: i64,
        stock: i32,
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
        if stock < 0 {
            return Err(DomainError::ValidationError(
                "Product stock cannot be negative".into(),
            ));
        }

        let now = Utc::now();

        Ok(Self {
            id,
            name,
            description,
            category_id,
            category_name: None,
            price_cents,
            stock,
            status: ProductStatus::Active,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn update(
        &mut self,
        name: String,
        description: Option<String>,
        category_id: Option<Uuid>,
        price_cents: i64,
        stock: i32,
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
        if stock < 0 {
            return Err(DomainError::ValidationError(
                "Product stock cannot be negative".into(),
            ));
        }

        self.name = name;
        self.description = description;
        self.category_id = category_id;
        self.price_cents = price_cents;
        self.stock = stock;
        self.updated_at = Utc::now();

        Ok(())
    }

    pub fn deactivate(&mut self) {
        self.status = ProductStatus::Inactive;
        self.updated_at = Utc::now();
    }
}
