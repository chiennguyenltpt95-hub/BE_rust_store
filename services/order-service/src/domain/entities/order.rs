use chrono::{DateTime, Utc};
use domain_core::error::DomainError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItem {
    pub id: Uuid,
    pub order_id: Uuid,
    pub product_id: Uuid,
    pub product_name: String,
    pub quantity: i32,
    pub unit_price_cents: i64,
    pub created_at: DateTime<Utc>,
}

impl OrderItem {
    pub fn create(
        order_id: Uuid,
        product_id: Uuid,
        product_name: String,
        quantity: i32,
        unit_price_cents: i64,
    ) -> Result<Self, DomainError> {
        if quantity <= 0 {
            return Err(DomainError::ValidationError("quantity must be greater than 0".into()));
        }
        if unit_price_cents < 0 {
            return Err(DomainError::ValidationError("unit_price_cents cannot be negative".into()));
        }

        Ok(Self {
            id: Uuid::new_v4(),
            order_id,
            product_id,
            product_name,
            quantity,
            unit_price_cents,
            created_at: Utc::now(),
        })
    }

    pub fn subtotal_cents(&self) -> i64 {
        self.quantity as i64 * self.unit_price_cents
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub user_id: Uuid,
    pub checkout_id: Uuid,
    pub cart_id: Uuid,
    pub idempotency_key: Option<String>,
    pub customer_email: String,
    pub customer_name: String,
    pub amount_cents: i64,
    pub currency: String,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Order {
    pub fn create(
        user_id: Uuid,
        checkout_id: Uuid,
        cart_id: Uuid,
        idempotency_key: Option<String>,
        customer_email: String,
        customer_name: String,
        amount_cents: i64,
        currency: String,
    ) -> Result<Self, DomainError> {
        if customer_email.trim().is_empty() {
            return Err(DomainError::ValidationError("customer_email cannot be empty".into()));
        }
        if customer_name.trim().is_empty() {
            return Err(DomainError::ValidationError("customer_name cannot be empty".into()));
        }
        if amount_cents <= 0 {
            return Err(DomainError::ValidationError("amount_cents must be greater than 0".into()));
        }

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            user_id,
            checkout_id,
            cart_id,
            idempotency_key,
            customer_email,
            customer_name,
            amount_cents,
            currency: currency.to_uppercase(),
            status: OrderStatus::Pending,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn confirm(&mut self) {
        self.status = OrderStatus::Confirmed;
        self.updated_at = Utc::now();
    }
}
