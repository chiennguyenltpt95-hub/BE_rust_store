use async_trait::async_trait;
use domain_core::error::DomainError;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::checkout::{CheckoutStatus, PaymentMethod};
use crate::domain::entities::{Checkout, PaymentTransaction};
use crate::domain::repositories::CheckoutRepository;

pub struct PgCheckoutRepository {
    pool: PgPool,
}

impl PgCheckoutRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct CheckoutRow {
    id: Uuid,
    user_id: Uuid,
    cart_id: Uuid,
    idempotency_key: Option<String>,
    amount_cents: i64,
    currency: String,
    status: String,
    payment_method: String,
    external_payment_id: Option<String>,
    checkout_url: Option<String>,
    failure_reason: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(sqlx::FromRow)]
struct PaymentTransactionRow {
    id: Uuid,
    checkout_id: Uuid,
    provider: String,
    provider_payment_id: String,
    amount_cents: i64,
    currency: String,
    status: String,
    raw_response: serde_json::Value,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl CheckoutRow {
    fn into_checkout(self) -> Checkout {
        let status = match self.status.as_str() {
            "paid" => CheckoutStatus::Paid,
            "failed" => CheckoutStatus::Failed,
            "cancelled" => CheckoutStatus::Cancelled,
            _ => CheckoutStatus::Pending,
        };

        let payment_method = match self.payment_method.as_str() {
            "stripe" => PaymentMethod::Stripe,
            "oxapay" => PaymentMethod::OxaPay,
            _ => PaymentMethod::Paypal,
        };

        Checkout {
            id: self.id,
            user_id: self.user_id,
            cart_id: self.cart_id,
            idempotency_key: self.idempotency_key,
            amount_cents: self.amount_cents,
            currency: self.currency,
            status,
            payment_method,
            external_payment_id: self.external_payment_id,
            checkout_url: self.checkout_url,
            failure_reason: self.failure_reason,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl From<PaymentTransactionRow> for PaymentTransaction {
    fn from(value: PaymentTransactionRow) -> Self {
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

#[async_trait]
impl CheckoutRepository for PgCheckoutRepository {
    async fn create_checkout(&self, checkout: &Checkout) -> Result<(), DomainError> {
        let status = format!("{:?}", checkout.status).to_lowercase();
        let payment_method = format!("{:?}", checkout.payment_method)
            .replace("OxaPay", "OxaPay")
            .to_lowercase();

        sqlx::query(
            r#"INSERT INTO checkouts
             (id, user_id, cart_id, idempotency_key, amount_cents, currency, status, payment_method, external_payment_id, checkout_url, failure_reason, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)"#,
        )
        .bind(checkout.id)
        .bind(checkout.user_id)
        .bind(checkout.cart_id)
         .bind(&checkout.idempotency_key)
        .bind(checkout.amount_cents)
        .bind(&checkout.currency)
        .bind(status)
        .bind(payment_method)
        .bind(&checkout.external_payment_id)
        .bind(&checkout.checkout_url)
        .bind(&checkout.failure_reason)
        .bind(checkout.created_at)
        .bind(checkout.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(())
    }

    async fn find_checkout_by_id(
        &self,
        checkout_id: Uuid,
    ) -> Result<Option<Checkout>, DomainError> {
        let row: Option<CheckoutRow> = sqlx::query_as(
            r#"SELECT id, user_id, cart_id, idempotency_key, amount_cents, currency, status, payment_method,
                      external_payment_id, checkout_url, failure_reason, created_at, updated_at
               FROM checkouts WHERE id = $1"#,
        )
        .bind(checkout_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(row.map(CheckoutRow::into_checkout))
    }

    async fn find_checkout_by_idempotency_key(
        &self,
        idempotency_key: &str,
    ) -> Result<Option<Checkout>, DomainError> {
        let row: Option<CheckoutRow> = sqlx::query_as(
            r#"SELECT id, user_id, cart_id, idempotency_key, amount_cents, currency, status, payment_method,
                      external_payment_id, checkout_url, failure_reason, created_at, updated_at
               FROM checkouts WHERE idempotency_key = $1"#,
        )
        .bind(idempotency_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(row.map(CheckoutRow::into_checkout))
    }

    async fn update_checkout(&self, checkout: &Checkout) -> Result<(), DomainError> {
        let status = format!("{:?}", checkout.status).to_lowercase();
        let payment_method = format!("{:?}", checkout.payment_method)
            .replace("OxaPay", "OxaPay")
            .to_lowercase();

        let res = sqlx::query(
            r#"UPDATE checkouts
               SET status = $2,
                   payment_method = $3,
                   external_payment_id = $4,
                   checkout_url = $5,
                   failure_reason = $6,
                   updated_at = $7
               WHERE id = $1"#,
        )
        .bind(checkout.id)
        .bind(status)
        .bind(payment_method)
        .bind(&checkout.external_payment_id)
        .bind(&checkout.checkout_url)
        .bind(&checkout.failure_reason)
        .bind(checkout.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        if res.rows_affected() == 0 {
            return Err(DomainError::NotFound(format!("Checkout {}", checkout.id)));
        }

        Ok(())
    }

    async fn create_transaction(&self, tx: &PaymentTransaction) -> Result<(), DomainError> {
        sqlx::query(
            r#"INSERT INTO payment_transactions
               (id, checkout_id, provider, provider_payment_id, amount_cents, currency, status, raw_response, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
        )
        .bind(tx.id)
        .bind(tx.checkout_id)
        .bind(&tx.provider)
        .bind(&tx.provider_payment_id)
        .bind(tx.amount_cents)
        .bind(&tx.currency)
        .bind(&tx.status)
        .bind(&tx.raw_response)
        .bind(tx.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(())
    }

    async fn transaction_exists(
        &self,
        provider: &str,
        provider_payment_id: &str,
    ) -> Result<bool, DomainError> {
        let count: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM payment_transactions WHERE provider = $1 AND provider_payment_id = $2"#,
        )
        .bind(provider)
        .bind(provider_payment_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(count.0 > 0)
    }

    async fn list_transactions_by_checkout(
        &self,
        checkout_id: Uuid,
    ) -> Result<Vec<PaymentTransaction>, DomainError> {
        let rows: Vec<PaymentTransactionRow> = sqlx::query_as(
            r#"SELECT id, checkout_id, provider, provider_payment_id, amount_cents, currency, status, raw_response, created_at
               FROM payment_transactions
               WHERE checkout_id = $1
               ORDER BY created_at DESC"#,
        )
        .bind(checkout_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(rows.into_iter().map(PaymentTransaction::from).collect())
    }
}
