use async_trait::async_trait;
use domain_core::error::DomainError;
use uuid::Uuid;

use crate::domain::entities::{Cart, CartItem};

#[async_trait]
pub trait CartRepository: Send + Sync {
    async fn create_cart(&self, cart: &Cart) -> Result<(), DomainError>;
    async fn find_cart_by_id(&self, cart_id: Uuid) -> Result<Option<Cart>, DomainError>;
    async fn find_active_cart_by_user_id(&self, user_id: Uuid)
        -> Result<Option<Cart>, DomainError>;
    async fn update_cart(&self, cart: &Cart) -> Result<(), DomainError>;

    async fn list_items(&self, cart_id: Uuid) -> Result<Vec<CartItem>, DomainError>;
    async fn upsert_item(&self, item: &CartItem) -> Result<(), DomainError>;
    async fn update_item_quantity(
        &self,
        cart_id: Uuid,
        item_id: Uuid,
        quantity: i32,
    ) -> Result<(), DomainError>;
    async fn remove_item(&self, cart_id: Uuid, item_id: Uuid) -> Result<(), DomainError>;
    async fn find_item_by_product_id(
        &self,
        cart_id: Uuid,
        product_id: Uuid,
    ) -> Result<Option<CartItem>, DomainError>;
}
