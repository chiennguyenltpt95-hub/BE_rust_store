use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::domain::entities::checkout::{
    Checkout, CheckoutStatus, PaymentMethod, PaymentTransaction,
};

#[derive(Debug, Clone, Serialize)]
pub struct PaymentTransactionView {
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

#[derive(Debug, Clone, Serialize)]
pub struct CheckoutView {
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
    pub transactions: Vec<PaymentTransactionView>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<PaymentTransaction> for PaymentTransactionView {
    fn from(value: PaymentTransaction) -> Self {
        Self {
            id: value.id,
            checkout_id: value.checkout_id,
            provider: value.provider,
            provider_payment_id: value.provider_payment_id,
            amount_cents: value.amount_cents,
            currency: value.currency,
            status: value.status,
            raw_response: value.raw_response,
            created_at: value.created_at,
        }
    }
}

impl CheckoutView {
    pub fn from_parts(value: Checkout, transactions: Vec<PaymentTransaction>) -> Self {
        Self {
            id: value.id,
            user_id: value.user_id,
            cart_id: value.cart_id,
            idempotency_key: value.idempotency_key,
            amount_cents: value.amount_cents,
            currency: value.currency,
            status: value.status,
            payment_method: value.payment_method,
            external_payment_id: value.external_payment_id,
            checkout_url: value.checkout_url,
            failure_reason: value.failure_reason,
            transactions: transactions
                .into_iter()
                .map(PaymentTransactionView::from)
                .collect(),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
