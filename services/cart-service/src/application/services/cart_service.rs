use domain_core::error::DomainError;
use std::sync::Arc;
use tracing::instrument;
use uuid::Uuid;
use validator::Validate;

use crate::application::commands::{AddItemCommand, CreateCartCommand, UpdateItemQuantityCommand};
use crate::application::ports::ProductPricingPort;
use crate::application::queries::CartView;
use crate::domain::entities::{Cart, CartItem};
use crate::domain::repositories::CartRepository;

pub struct CartAppService {
    cart_repo: Arc<dyn CartRepository>,
    product_pricing: Arc<dyn ProductPricingPort>,
}

impl CartAppService {
    pub fn new(
        cart_repo: Arc<dyn CartRepository>,
        product_pricing: Arc<dyn ProductPricingPort>,
    ) -> Self {
        Self {
            cart_repo,
            product_pricing,
        }
    }

    #[instrument(skip(self, cmd))]
    pub async fn create_cart(&self, cmd: CreateCartCommand) -> Result<Uuid, DomainError> {
        let existing = self.cart_repo.find_active_cart_by_user_id(cmd.user_id).await?;
        if let Some(cart) = existing {
            return Ok(cart.id);
        }

        let cart = Cart::create(cmd.user_id);
        let cart_id = cart.id;
        self.cart_repo.create_cart(&cart).await?;
        Ok(cart_id)
    }

    #[instrument(skip(self))]
    pub async fn get_cart(&self, cart_id: Uuid) -> Result<CartView, DomainError> {
        let cart = self
            .cart_repo
            .find_cart_by_id(cart_id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Cart {}", cart_id)))?;
        let items = self.cart_repo.list_items(cart_id).await?;
        Ok(CartView::from_parts(cart, items))
    }

    #[instrument(skip(self))]
    pub async fn get_active_cart_by_user(&self, user_id: Uuid) -> Result<CartView, DomainError> {
        let cart = self
            .cart_repo
            .find_active_cart_by_user_id(user_id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Active cart for user {}", user_id)))?;
        let items = self.cart_repo.list_items(cart.id).await?;
        Ok(CartView::from_parts(cart, items))
    }

    #[instrument(skip(self, cmd))]
    pub async fn add_item(&self, cart_id: Uuid, cmd: AddItemCommand) -> Result<(), DomainError> {
        cmd.validate()
            .map_err(|e| DomainError::ValidationError(e.to_string()))?;

        let unit_price_cents = self
            .product_pricing
            .get_product_price_cents(cmd.product_id)
            .await?;

        let mut cart = self
            .cart_repo
            .find_cart_by_id(cart_id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Cart {}", cart_id)))?;

        let existing_item = self
            .cart_repo
            .find_item_by_product_id(cart_id, cmd.product_id)
            .await?;

        let item = if let Some(mut current) = existing_item {
            current.quantity += cmd.quantity;
            current.unit_price_cents = unit_price_cents;
            current.updated_at = chrono::Utc::now();
            current
        } else {
            CartItem::create(cart_id, cmd.product_id, cmd.quantity, unit_price_cents)?
        };

        self.cart_repo.upsert_item(&item).await?;
        self.refresh_total(&mut cart).await?;
        Ok(())
    }

    #[instrument(skip(self, cmd))]
    pub async fn update_item_quantity(
        &self,
        cart_id: Uuid,
        item_id: Uuid,
        cmd: UpdateItemQuantityCommand,
    ) -> Result<(), DomainError> {
        cmd.validate()
            .map_err(|e| DomainError::ValidationError(e.to_string()))?;

        let mut cart = self
            .cart_repo
            .find_cart_by_id(cart_id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Cart {}", cart_id)))?;

        self.cart_repo
            .update_item_quantity(cart_id, item_id, cmd.quantity)
            .await?;

        self.refresh_total(&mut cart).await?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn remove_item(&self, cart_id: Uuid, item_id: Uuid) -> Result<(), DomainError> {
        let mut cart = self
            .cart_repo
            .find_cart_by_id(cart_id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Cart {}", cart_id)))?;

        self.cart_repo.remove_item(cart_id, item_id).await?;
        self.refresh_total(&mut cart).await?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn checkout_cart(&self, cart_id: Uuid) -> Result<(), DomainError> {
        let mut cart = self
            .cart_repo
            .find_cart_by_id(cart_id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Cart {}", cart_id)))?;

        let items = self.cart_repo.list_items(cart_id).await?;
        if items.is_empty() {
            return Err(DomainError::BusinessRuleViolation(
                "Cannot checkout an empty cart".into(),
            ));
        }

        cart.checkout()?;
        self.cart_repo.update_cart(&cart).await
    }

    async fn refresh_total(&self, cart: &mut Cart) -> Result<(), DomainError> {
        let items = self.cart_repo.list_items(cart.id).await?;
        let total_cents = items.iter().map(|i| i.subtotal_cents()).sum();
        cart.set_total(total_cents);
        self.cart_repo.update_cart(cart).await
    }
}
