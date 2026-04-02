use chrono::{DateTime, Utc};
use domain_core::error::DomainError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CheckoutStatus {
    Pending,
    Paid,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PaymentMethod {
    Paypal,
    Stripe,
    OxaPay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkout {
    pub id: Uuid,
    pub user_id: Uuid,
    pub cart_id: Uuid,
    pub idempotency_key: Option<String>,
    pub amount_cents: i64,
    pub currency: String,
    pub status: CheckoutStatus,
    pub payment_method: PaymentMethod,
    pub external_payment_id: Option<String>,
    pub checkout_url: Option<String>,
    pub failure_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Checkout {
    pub fn create(
        user_id: Uuid,
        cart_id: Uuid,
        idempotency_key: Option<String>,
        amount_cents: i64,
        currency: String,
        payment_method: PaymentMethod,
    ) -> Result<Self, DomainError> {
        if amount_cents <= 0 {
            return Err(DomainError::ValidationError(
                "amount_cents must be greater than 0".into(),
            ));
        }
        if currency.trim().is_empty() {
            return Err(DomainError::ValidationError("currency cannot be empty".into()));
        }

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            user_id,
            cart_id,
            idempotency_key,
            amount_cents,
            currency: currency.to_uppercase(),
            status: CheckoutStatus::Pending,
            payment_method,
            external_payment_id: None,
            checkout_url: None,
            failure_reason: None,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn set_payment_session(&mut self, external_payment_id: String, checkout_url: String) {
        self.external_payment_id = Some(external_payment_id);
        self.checkout_url = Some(checkout_url);
        self.updated_at = Utc::now();
    }

    pub fn mark_paid(&mut self) {
        self.status = CheckoutStatus::Paid;
        self.failure_reason = None;
        self.updated_at = Utc::now();
    }

    pub fn mark_failed(&mut self, reason: String) {
        self.status = CheckoutStatus::Failed;
        self.failure_reason = Some(reason);
        self.updated_at = Utc::now();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentTransaction {
    pub id: Uuid,
    pub checkout_id: Uuid,
    pub provider: String,
    pub provider_payment_id: String,
    pub amount_cents: i64,
    pub currency: String,
    pub status: String,
    pub raw_response: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

impl PaymentTransaction {
    pub fn create(
        checkout_id: Uuid,
        provider: String,
        provider_payment_id: String,
        amount_cents: i64,
        currency: String,
        status: String,
        raw_response: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            checkout_id,
            provider,
            provider_payment_id,
            amount_cents,
            currency,
            status,
            raw_response,
            created_at: Utc::now(),
        }
    }
}
