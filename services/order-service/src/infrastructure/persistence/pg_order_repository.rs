use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain_core::error::DomainError;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::{Order, OrderItem, OrderStatus, OutboxMessage, OutboxStats};
use crate::domain::repositories::OrderRepository;

pub struct PgOrderRepository {
    pool: PgPool,
}

impl PgOrderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct OrderRow {
    id: Uuid,
    user_id: Uuid,
    checkout_id: Uuid,
    cart_id: Uuid,
    idempotency_key: Option<String>,
    customer_email: String,
    customer_name: String,
    amount_cents: i64,
    currency: String,
    status: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(sqlx::FromRow)]
struct OrderItemRow {
    id: Uuid,
    order_id: Uuid,
    product_id: Uuid,
    product_name: String,
    quantity: i32,
    unit_price_cents: i64,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(sqlx::FromRow)]
struct OutboxRow {
    id: Uuid,
    aggregate_type: String,
    aggregate_id: Uuid,
    event_type: String,
    payload: serde_json::Value,
    status: String,
    attempts: i32,
    next_retry_at: chrono::DateTime<chrono::Utc>,
    last_error: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl OrderRow {
    fn into_order(self) -> Order {
        let status = match self.status.as_str() {
            "confirmed" => OrderStatus::Confirmed,
            "cancelled" => OrderStatus::Cancelled,
            _ => OrderStatus::Pending,
        };

        Order {
            id: self.id,
            user_id: self.user_id,
            checkout_id: self.checkout_id,
            cart_id: self.cart_id,
            idempotency_key: self.idempotency_key,
            customer_email: self.customer_email,
            customer_name: self.customer_name,
            amount_cents: self.amount_cents,
            currency: self.currency,
            status,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl From<OrderItemRow> for OrderItem {
    fn from(value: OrderItemRow) -> Self {
        Self {
            id: value.id,
            order_id: value.order_id,
            product_id: value.product_id,
            product_name: value.product_name,
            quantity: value.quantity,
            unit_price_cents: value.unit_price_cents,
            created_at: value.created_at,
        }
    }
}

impl From<OutboxRow> for OutboxMessage {
    fn from(value: OutboxRow) -> Self {
        Self {
            id: value.id,
            aggregate_type: value.aggregate_type,
            aggregate_id: value.aggregate_id,
            event_type: value.event_type,
            payload: value.payload,
            status: value.status,
            attempts: value.attempts,
            next_retry_at: value.next_retry_at,
            last_error: value.last_error,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[async_trait]
impl OrderRepository for PgOrderRepository {
    async fn create_order_with_outbox(
        &self,
        order: &Order,
        items: &[OrderItem],
        outbox_event_type: &str,
        outbox_payload: &serde_json::Value,
    ) -> Result<(), DomainError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        let status = format!("{:?}", order.status).to_lowercase();
        sqlx::query(
            r#"INSERT INTO orders
               (id, user_id, checkout_id, cart_id, idempotency_key, customer_email, customer_name, amount_cents, currency, status, created_at, updated_at)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)"#,
        )
        .bind(order.id)
        .bind(order.user_id)
        .bind(order.checkout_id)
        .bind(order.cart_id)
        .bind(&order.idempotency_key)
        .bind(&order.customer_email)
        .bind(&order.customer_name)
        .bind(order.amount_cents)
        .bind(&order.currency)
        .bind(status)
        .bind(order.created_at)
        .bind(order.updated_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        for item in items {
            sqlx::query(
                r#"INSERT INTO order_items
                   (id, order_id, product_id, product_name, quantity, unit_price_cents, created_at)
                   VALUES ($1,$2,$3,$4,$5,$6,$7)"#,
            )
            .bind(item.id)
            .bind(item.order_id)
            .bind(item.product_id)
            .bind(&item.product_name)
            .bind(item.quantity)
            .bind(item.unit_price_cents)
            .bind(item.created_at)
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;
        }

        sqlx::query(
            r#"INSERT INTO outbox_messages
               (id, aggregate_type, aggregate_id, event_type, payload, status, attempts, next_retry_at, created_at, updated_at)
               VALUES ($1,$2,$3,$4,$5,'pending',0,NOW(),NOW(),NOW())"#,
        )
        .bind(Uuid::new_v4())
        .bind("order")
        .bind(order.id)
        .bind(outbox_event_type)
        .bind(outbox_payload)
        .execute(&mut *tx)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))
    }

    async fn find_order_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<(Order, Vec<OrderItem>)>, DomainError> {
        self.load_one("SELECT id, user_id, checkout_id, cart_id, idempotency_key, customer_email, customer_name, amount_cents, currency, status, created_at, updated_at FROM orders WHERE id = $1", id)
            .await
    }

    async fn find_by_checkout_id(
        &self,
        checkout_id: Uuid,
    ) -> Result<Option<(Order, Vec<OrderItem>)>, DomainError> {
        self.load_one("SELECT id, user_id, checkout_id, cart_id, idempotency_key, customer_email, customer_name, amount_cents, currency, status, created_at, updated_at FROM orders WHERE checkout_id = $1", checkout_id)
            .await
    }

    async fn find_by_idempotency_key(
        &self,
        key: &str,
    ) -> Result<Option<(Order, Vec<OrderItem>)>, DomainError> {
        let row: Option<OrderRow> = sqlx::query_as("SELECT id, user_id, checkout_id, cart_id, idempotency_key, customer_email, customer_name, amount_cents, currency, status, created_at, updated_at FROM orders WHERE idempotency_key = $1")
            .bind(key)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        if let Some(order_row) = row {
            let items = self.load_items(order_row.id).await?;
            return Ok(Some((order_row.into_order(), items)));
        }
        Ok(None)
    }

    async fn list_orders_by_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<(Order, Vec<OrderItem>)>, DomainError> {
        let rows: Vec<OrderRow> = sqlx::query_as("SELECT id, user_id, checkout_id, cart_id, idempotency_key, customer_email, customer_name, amount_cents, currency, status, created_at, updated_at FROM orders WHERE user_id = $1 ORDER BY created_at DESC")
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        let mut result = Vec::with_capacity(rows.len());
        for row in rows {
            let items = self.load_items(row.id).await?;
            result.push((row.into_order(), items));
        }
        Ok(result)
    }

    async fn dequeue_outbox_pending(&self, limit: i64) -> Result<Vec<OutboxMessage>, DomainError> {
        let rows: Vec<OutboxRow> = sqlx::query_as(
            r#"
            WITH picked AS (
                SELECT id
                FROM outbox_messages
                WHERE status IN ('pending', 'failed')
                  AND next_retry_at <= NOW()
                ORDER BY created_at ASC
                LIMIT $1
                FOR UPDATE SKIP LOCKED
            )
            UPDATE outbox_messages m
            SET status = 'processing', updated_at = NOW()
            WHERE m.id IN (SELECT id FROM picked)
            RETURNING m.id, m.aggregate_type, m.aggregate_id, m.event_type, m.payload, m.status,
                      m.attempts, m.next_retry_at, m.last_error, m.created_at, m.updated_at
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(rows.into_iter().map(OutboxMessage::from).collect())
    }

    async fn mark_outbox_sent(&self, id: Uuid) -> Result<(), DomainError> {
        sqlx::query(
            "UPDATE outbox_messages SET status = 'sent', last_error = NULL, updated_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;
        Ok(())
    }

    async fn mark_outbox_failed(
        &self,
        id: Uuid,
        error: &str,
        next_retry_at: DateTime<Utc>,
    ) -> Result<(), DomainError> {
        sqlx::query(
            "UPDATE outbox_messages SET status = 'failed', attempts = attempts + 1, last_error = $2, next_retry_at = $3, updated_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .bind(error)
        .bind(next_retry_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;
        Ok(())
    }

    async fn mark_outbox_dead_letter(&self, id: Uuid, error: &str) -> Result<(), DomainError> {
        sqlx::query(
            "UPDATE outbox_messages SET status = 'dead_letter', attempts = attempts + 1, last_error = $2, updated_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .bind(error)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;
        Ok(())
    }

    async fn get_outbox_stats(&self) -> Result<OutboxStats, DomainError> {
        #[derive(sqlx::FromRow)]
        struct StatsRow {
            pending: i64,
            processing: i64,
            failed: i64,
            dead_letter: i64,
            sent: i64,
        }

        let row: StatsRow = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE status = 'pending')::BIGINT AS pending,
                COUNT(*) FILTER (WHERE status = 'processing')::BIGINT AS processing,
                COUNT(*) FILTER (WHERE status = 'failed')::BIGINT AS failed,
                COUNT(*) FILTER (WHERE status = 'dead_letter')::BIGINT AS dead_letter,
                COUNT(*) FILTER (WHERE status = 'sent')::BIGINT AS sent
            FROM outbox_messages
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(OutboxStats {
            pending: row.pending,
            processing: row.processing,
            failed: row.failed,
            dead_letter: row.dead_letter,
            sent: row.sent,
        })
    }

    async fn list_outbox_messages(
        &self,
        status: Option<&str>,
        limit: i64,
    ) -> Result<Vec<OutboxMessage>, DomainError> {
        let rows: Vec<OutboxRow> = match status {
            Some(status) => {
                sqlx::query_as(
                    r#"
                    SELECT id, aggregate_type, aggregate_id, event_type, payload, status,
                           attempts, next_retry_at, last_error, created_at, updated_at
                    FROM outbox_messages
                    WHERE status = $1
                    ORDER BY created_at DESC
                    LIMIT $2
                    "#,
                )
                .bind(status)
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
            None => {
                sqlx::query_as(
                    r#"
                    SELECT id, aggregate_type, aggregate_id, event_type, payload, status,
                           attempts, next_retry_at, last_error, created_at, updated_at
                    FROM outbox_messages
                    ORDER BY created_at DESC
                    LIMIT $1
                    "#,
                )
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
        }
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(rows.into_iter().map(OutboxMessage::from).collect())
    }
}

impl PgOrderRepository {
    async fn load_one(
        &self,
        sql: &str,
        id: Uuid,
    ) -> Result<Option<(Order, Vec<OrderItem>)>, DomainError> {
        let row: Option<OrderRow> = sqlx::query_as(sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        if let Some(order_row) = row {
            let items = self.load_items(order_row.id).await?;
            return Ok(Some((order_row.into_order(), items)));
        }
        Ok(None)
    }

    async fn load_items(&self, order_id: Uuid) -> Result<Vec<OrderItem>, DomainError> {
        let rows: Vec<OrderItemRow> = sqlx::query_as(
            "SELECT id, order_id, product_id, product_name, quantity, unit_price_cents, created_at FROM order_items WHERE order_id = $1 ORDER BY created_at ASC",
        )
        .bind(order_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(rows.into_iter().map(OrderItem::from).collect())
    }
}
