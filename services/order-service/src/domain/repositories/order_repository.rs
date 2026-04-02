use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain_core::error::DomainError;
use uuid::Uuid;

use crate::domain::entities::{Order, OrderItem, OutboxMessage, OutboxStats};

#[async_trait]
pub trait OrderRepository: Send + Sync {
    async fn create_order_with_outbox(
        &self,
        order: &Order,
        items: &[OrderItem],
        outbox_event_type: &str,
        outbox_payload: &serde_json::Value,
    ) -> Result<(), DomainError>;
    async fn find_order_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<(Order, Vec<OrderItem>)>, DomainError>;
    async fn find_by_checkout_id(
        &self,
        checkout_id: Uuid,
    ) -> Result<Option<(Order, Vec<OrderItem>)>, DomainError>;
    async fn find_by_idempotency_key(
        &self,
        key: &str,
    ) -> Result<Option<(Order, Vec<OrderItem>)>, DomainError>;
    async fn list_orders_by_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<(Order, Vec<OrderItem>)>, DomainError>;
    async fn dequeue_outbox_pending(&self, limit: i64) -> Result<Vec<OutboxMessage>, DomainError>;
    async fn mark_outbox_sent(&self, id: Uuid) -> Result<(), DomainError>;
    async fn mark_outbox_failed(
        &self,
        id: Uuid,
        error: &str,
        next_retry_at: DateTime<Utc>,
    ) -> Result<(), DomainError>;
    async fn mark_outbox_dead_letter(&self, id: Uuid, error: &str) -> Result<(), DomainError>;
    async fn get_outbox_stats(&self) -> Result<OutboxStats, DomainError>;
    async fn list_outbox_messages(
        &self,
        status: Option<&str>,
        limit: i64,
    ) -> Result<Vec<OutboxMessage>, DomainError>;
}
