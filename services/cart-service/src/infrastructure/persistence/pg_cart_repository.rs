use async_trait::async_trait;
use domain_core::error::DomainError;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::{Cart, CartItem, CartStatus};
use crate::domain::repositories::CartRepository;

pub struct PgCartRepository {
    pool: PgPool,
}

impl PgCartRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct CartRow {
    id: Uuid,
    user_id: Uuid,
    status: String,
    total_cents: i64,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(sqlx::FromRow)]
struct CartItemRow {
    id: Uuid,
    cart_id: Uuid,
    product_id: Uuid,
    quantity: i32,
    unit_price_cents: i64,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl CartRow {
    fn into_cart(self) -> Cart {
        let status = match self.status.as_str() {
            "checked_out" => CartStatus::CheckedOut,
            "abandoned" => CartStatus::Abandoned,
            _ => CartStatus::Active,
        };

        Cart {
            id: self.id,
            user_id: self.user_id,
            status,
            total_cents: self.total_cents,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl CartItemRow {
    fn into_item(self) -> CartItem {
        CartItem {
            id: self.id,
            cart_id: self.cart_id,
            product_id: self.product_id,
            quantity: self.quantity,
            unit_price_cents: self.unit_price_cents,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[async_trait]
impl CartRepository for PgCartRepository {
    async fn create_cart(&self, cart: &Cart) -> Result<(), DomainError> {
        let status = format!("{:?}", cart.status).to_lowercase();
        sqlx::query(
            r#"INSERT INTO carts (id, user_id, status, total_cents, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6)"#,
        )
        .bind(cart.id)
        .bind(cart.user_id)
        .bind(status)
        .bind(cart.total_cents)
        .bind(cart.created_at)
        .bind(cart.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;
        Ok(())
    }

    async fn find_cart_by_id(&self, cart_id: Uuid) -> Result<Option<Cart>, DomainError> {
        let row: Option<CartRow> = sqlx::query_as(
            r#"SELECT id, user_id, status, total_cents, created_at, updated_at
               FROM carts WHERE id = $1"#,
        )
        .bind(cart_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;
        Ok(row.map(CartRow::into_cart))
    }

    async fn find_active_cart_by_user_id(&self, user_id: Uuid) -> Result<Option<Cart>, DomainError> {
        let row: Option<CartRow> = sqlx::query_as(
            r#"SELECT id, user_id, status, total_cents, created_at, updated_at
               FROM carts WHERE user_id = $1 AND status = 'active'
               ORDER BY created_at DESC LIMIT 1"#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;
        Ok(row.map(CartRow::into_cart))
    }

    async fn update_cart(&self, cart: &Cart) -> Result<(), DomainError> {
        let status = format!("{:?}", cart.status).to_lowercase();
        let res = sqlx::query(
            r#"UPDATE carts
               SET status = $2, total_cents = $3, updated_at = $4
               WHERE id = $1"#,
        )
        .bind(cart.id)
        .bind(status)
        .bind(cart.total_cents)
        .bind(cart.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        if res.rows_affected() == 0 {
            return Err(DomainError::NotFound(format!("Cart {}", cart.id)));
        }

        Ok(())
    }

    async fn list_items(&self, cart_id: Uuid) -> Result<Vec<CartItem>, DomainError> {
        let rows: Vec<CartItemRow> = sqlx::query_as(
            r#"SELECT id, cart_id, product_id, quantity, unit_price_cents, created_at, updated_at
               FROM cart_items WHERE cart_id = $1 ORDER BY created_at ASC"#,
        )
        .bind(cart_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(rows.into_iter().map(CartItemRow::into_item).collect())
    }

    async fn upsert_item(&self, item: &CartItem) -> Result<(), DomainError> {
        sqlx::query(
            r#"INSERT INTO cart_items (id, cart_id, product_id, quantity, unit_price_cents, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               ON CONFLICT (cart_id, product_id)
               DO UPDATE SET quantity = EXCLUDED.quantity,
                             unit_price_cents = EXCLUDED.unit_price_cents,
                             updated_at = EXCLUDED.updated_at"#,
        )
        .bind(item.id)
        .bind(item.cart_id)
        .bind(item.product_id)
        .bind(item.quantity)
        .bind(item.unit_price_cents)
        .bind(item.created_at)
        .bind(item.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(())
    }

    async fn update_item_quantity(&self, cart_id: Uuid, item_id: Uuid, quantity: i32) -> Result<(), DomainError> {
        let res = sqlx::query(
            r#"UPDATE cart_items SET quantity = $3, updated_at = NOW()
               WHERE cart_id = $1 AND id = $2"#,
        )
        .bind(cart_id)
        .bind(item_id)
        .bind(quantity)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        if res.rows_affected() == 0 {
            return Err(DomainError::NotFound(format!("Cart item {}", item_id)));
        }

        Ok(())
    }

    async fn remove_item(&self, cart_id: Uuid, item_id: Uuid) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM cart_items WHERE cart_id = $1 AND id = $2")
            .bind(cart_id)
            .bind(item_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;
        Ok(())
    }

    async fn find_item_by_product_id(&self, cart_id: Uuid, product_id: Uuid) -> Result<Option<CartItem>, DomainError> {
        let row: Option<CartItemRow> = sqlx::query_as(
            r#"SELECT id, cart_id, product_id, quantity, unit_price_cents, created_at, updated_at
               FROM cart_items WHERE cart_id = $1 AND product_id = $2"#,
        )
        .bind(cart_id)
        .bind(product_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(row.map(CartItemRow::into_item))
    }
}
