use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::domain::entities::{Order, OrderItem, OrderStatus};

#[derive(Debug, Clone, Serialize)]
pub struct OrderItemView {
    pub id: Uuid,
    pub product_id: Uuid,
    pub product_name: String,
    pub quantity: i32,
    pub unit_price_cents: i64,
    pub subtotal_cents: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OrderView {
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
    pub items: Vec<OrderItemView>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<OrderItem> for OrderItemView {
    fn from(value: OrderItem) -> Self {
        let subtotal_cents = value.subtotal_cents();
        Self {
            id: value.id,
            product_id: value.product_id,
            product_name: value.product_name,
            quantity: value.quantity,
            unit_price_cents: value.unit_price_cents,
            subtotal_cents,
            created_at: value.created_at,
        }
    }
}

impl OrderView {
    pub fn from_parts(order: Order, items: Vec<OrderItem>) -> Self {
        Self {
            id: order.id,
            user_id: order.user_id,
            checkout_id: order.checkout_id,
            cart_id: order.cart_id,
            idempotency_key: order.idempotency_key,
            customer_email: order.customer_email,
            customer_name: order.customer_name,
            amount_cents: order.amount_cents,
            currency: order.currency,
            status: order.status,
            items: items.into_iter().map(OrderItemView::from).collect(),
            created_at: order.created_at,
            updated_at: order.updated_at,
        }
    }
}
