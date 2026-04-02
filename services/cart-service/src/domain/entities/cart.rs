use chrono::{DateTime, Utc};
use domain_core::error::DomainError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CartStatus {
    Active,
    CheckedOut,
    Abandoned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartItem {
    pub id: Uuid,
    pub cart_id: Uuid,
    pub product_id: Uuid,
    pub quantity: i32,
    pub unit_price_cents: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl CartItem {
    pub fn create(
        cart_id: Uuid,
        product_id: Uuid,
        quantity: i32,
        unit_price_cents: i64,
    ) -> Result<Self, DomainError> {
        if quantity <= 0 {
            return Err(DomainError::ValidationError(
                "Quantity must be greater than 0".into(),
            ));
        }
        if unit_price_cents < 0 {
            return Err(DomainError::ValidationError(
                "Unit price cannot be negative".into(),
            ));
        }

        let now = Utc::now();

        Ok(Self {
            id: Uuid::new_v4(),
            cart_id,
            product_id,
            quantity,
            unit_price_cents,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn subtotal_cents(&self) -> i64 {
        self.quantity as i64 * self.unit_price_cents
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cart {
    pub id: Uuid,
    pub user_id: Uuid,
    pub status: CartStatus,
    pub total_cents: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Cart {
    pub fn create(user_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            status: CartStatus::Active,
            total_cents: 0,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn checkout(&mut self) -> Result<(), DomainError> {
        if self.status != CartStatus::Active {
            return Err(DomainError::BusinessRuleViolation(
                "Only active cart can be checked out".into(),
            ));
        }
        self.status = CartStatus::CheckedOut;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn set_total(&mut self, total_cents: i64) {
        self.total_cents = total_cents;
        self.updated_at = Utc::now();
    }
}
