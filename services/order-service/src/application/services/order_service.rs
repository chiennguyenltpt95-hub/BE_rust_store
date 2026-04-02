use domain_core::error::DomainError;
use std::sync::Arc;
use tracing::instrument;
use uuid::Uuid;
use validator::Validate;

use crate::application::commands::CreateOrderCommand;
use crate::application::ports::{CheckoutReaderPort, MailMessage};
use crate::application::queries::OrderView;
use crate::domain::entities::{Order, OrderItem};
use crate::domain::repositories::OrderRepository;

pub struct OrderAppService {
    repo: Arc<dyn OrderRepository>,
    checkout_reader: Arc<dyn CheckoutReaderPort>,
}

impl OrderAppService {
    pub fn new(
        repo: Arc<dyn OrderRepository>,
        checkout_reader: Arc<dyn CheckoutReaderPort>,
    ) -> Self {
        Self {
            repo,
            checkout_reader,
        }
    }

    #[instrument(skip(self, cmd))]
    pub async fn create_order(&self, cmd: CreateOrderCommand) -> Result<OrderView, DomainError> {
        cmd.validate()
            .map_err(|e| DomainError::ValidationError(e.to_string()))?;

        if let Some(key) = &cmd.idempotency_key {
            if let Some((order, items)) = self.repo.find_by_idempotency_key(key).await? {
                return Ok(OrderView::from_parts(order, items));
            }
        }

        if let Some((existing_order, existing_items)) =
            self.repo.find_by_checkout_id(cmd.checkout_id).await?
        {
            return Ok(OrderView::from_parts(existing_order, existing_items));
        }

        let checkout = self.checkout_reader.get_checkout(cmd.checkout_id).await?;
        if checkout.status.to_lowercase() != "paid" {
            return Err(DomainError::BusinessRuleViolation(
                "Checkout is not paid yet".into(),
            ));
        }

        let items: Vec<OrderItem> = cmd
            .items
            .iter()
            .map(|item| {
                OrderItem::create(
                    Uuid::nil(),
                    item.product_id,
                    item.product_name.clone(),
                    item.quantity,
                    item.unit_price_cents,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;

        let items_total: i64 = items.iter().map(|i| i.subtotal_cents()).sum();
        if items_total != checkout.amount_cents {
            return Err(DomainError::BusinessRuleViolation(format!(
                "Order amount mismatch: items={} checkout={}",
                items_total, checkout.amount_cents
            )));
        }

        if cmd.currency.to_uppercase() != checkout.currency.to_uppercase() {
            return Err(DomainError::BusinessRuleViolation(
                "Order currency does not match checkout currency".into(),
            ));
        }

        let mut order = Order::create(
            cmd.user_id,
            cmd.checkout_id,
            cmd.cart_id,
            cmd.idempotency_key,
            cmd.customer_email,
            cmd.customer_name,
            checkout.amount_cents,
            checkout.currency,
        )?;
        order.confirm();

        let bound_items = items
            .into_iter()
            .map(|mut i| {
                i.order_id = order.id;
                i
            })
            .collect::<Vec<_>>();

        let mail_event = MailMessage {
            to: order.customer_email.clone(),
            to_name: Some(order.customer_name.clone()),
            subject: format!("Order #{} confirmation", order.id),
            template_name: "order_confirmation".into(),
            context: serde_json::json!({
                "full_name": order.customer_name,
                "order_id": order.id,
            }),
        };

        self.repo
            .create_order_with_outbox(
                &order,
                &bound_items,
                "order.confirmed.mail",
                &serde_json::to_value(mail_event)
                    .map_err(|e| DomainError::InfrastructureError(e.to_string()))?,
            )
            .await?;

        Ok(OrderView::from_parts(order, bound_items))
    }

    pub async fn get_order(&self, id: Uuid) -> Result<OrderView, DomainError> {
        let (order, items) = self
            .repo
            .find_order_by_id(id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Order {}", id)))?;
        Ok(OrderView::from_parts(order, items))
    }

    pub async fn list_orders_by_user(&self, user_id: Uuid) -> Result<Vec<OrderView>, DomainError> {
        let rows = self.repo.list_orders_by_user(user_id).await?;
        Ok(rows
            .into_iter()
            .map(|(order, items)| OrderView::from_parts(order, items))
            .collect())
    }
}
