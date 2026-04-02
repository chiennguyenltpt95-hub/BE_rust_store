use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::domain::entities::{Cart, CartItem, CartStatus};

#[derive(Debug, Clone, Serialize)]
pub struct CartItemView {
    pub id: Uuid,
    pub product_id: Uuid,
    pub quantity: i32,
    pub unit_price_cents: i64,
    pub subtotal_cents: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CartView {
    pub id: Uuid,
    pub user_id: Uuid,
    pub status: CartStatus,
    pub total_cents: i64,
    pub items: Vec<CartItemView>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<CartItem> for CartItemView {
    fn from(value: CartItem) -> Self {
        Self {
            id: value.id,
            product_id: value.product_id,
            quantity: value.quantity,
            unit_price_cents: value.unit_price_cents,
            subtotal_cents: value.subtotal_cents(),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl CartView {
    pub fn from_parts(cart: Cart, items: Vec<CartItem>) -> Self {
        Self {
            id: cart.id,
            user_id: cart.user_id,
            status: cart.status,
            total_cents: cart.total_cents,
            items: items.into_iter().map(CartItemView::from).collect(),
            created_at: cart.created_at,
            updated_at: cart.updated_at,
        }
    }
}
