use domain_core::error::DomainError;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::sync::Arc;
use tracing::instrument;
use uuid::Uuid;
use validator::Validate;

use crate::application::commands::{
    CreateCheckoutCommand, MarkFailedCommand, MarkPaidCommand, ProviderWebhookCommand,
};
use crate::application::ports::{
    CartReaderPort, NotificationSenderPort, PaymentGatewayFactoryPort, PaymentRequest,
};
use crate::application::queries::CheckoutView;
use crate::domain::entities::checkout::{Checkout, PaymentMethod, PaymentTransaction};
use crate::domain::repositories::CheckoutRepository;

pub struct CheckoutAppService {
    repo: Arc<dyn CheckoutRepository>,
    payment_factory: Arc<dyn PaymentGatewayFactoryPort>,
    cart_reader: Arc<dyn CartReaderPort>,
    notification_sender: Arc<dyn NotificationSenderPort>,
    paypal_webhook_secret: String,
    stripe_webhook_secret: String,
    oxapay_webhook_secret: String,
    success_url: String,
    cancel_url: String,
}

impl CheckoutAppService {
    pub fn new(
        repo: Arc<dyn CheckoutRepository>,
        payment_factory: Arc<dyn PaymentGatewayFactoryPort>,
        cart_reader: Arc<dyn CartReaderPort>,
        notification_sender: Arc<dyn NotificationSenderPort>,
        paypal_webhook_secret: String,
        stripe_webhook_secret: String,
        oxapay_webhook_secret: String,
        success_url: String,
        cancel_url: String,
    ) -> Self {
        Self {
            repo,
            payment_factory,
            cart_reader,
            notification_sender,
            paypal_webhook_secret,
            stripe_webhook_secret,
            oxapay_webhook_secret,
            success_url,
            cancel_url,
        }
    }

    #[instrument(skip(self, cmd))]
    pub async fn create_checkout(
        &self,
        cmd: CreateCheckoutCommand,
    ) -> Result<CheckoutView, DomainError> {
        cmd.validate()
            .map_err(|e| DomainError::ValidationError(e.to_string()))?;

        if let Some(key) = &cmd.idempotency_key {
            if let Some(existing) = self.repo.find_checkout_by_idempotency_key(key).await? {
                return self.build_checkout_view(existing).await;
            }
        }

        let payment_method = parse_payment_method(&cmd.payment_method)?;
        let amount_cents = self.cart_reader.get_cart_total_cents(cmd.cart_id).await?;
        let mut checkout = Checkout::create(
            cmd.user_id,
            cmd.cart_id,
            cmd.idempotency_key,
            amount_cents,
            cmd.currency,
            payment_method.clone(),
        )?;

        self.repo.create_checkout(&checkout).await?;

        let gateway = self.payment_factory.get_gateway(&payment_method)?;
        let payment_req = PaymentRequest {
            checkout_id: checkout.id,
            amount_cents: checkout.amount_cents,
            currency: checkout.currency.clone(),
            description: cmd
                .description
                .unwrap_or_else(|| format!("Checkout {}", checkout.id)),
            success_url: self.success_url.clone(),
            cancel_url: self.cancel_url.clone(),
        };

        match gateway.create_payment(&payment_req).await {
            Ok(result) => {
                checkout.set_payment_session(
                    result.provider_payment_id.clone(),
                    result.checkout_url.clone(),
                );
                self.repo.update_checkout(&checkout).await?;

                let tx = PaymentTransaction::create(
                    checkout.id,
                    result.provider,
                    result.provider_payment_id,
                    checkout.amount_cents,
                    checkout.currency.clone(),
                    result.status,
                    result.raw_response,
                );
                self.repo.create_transaction(&tx).await?;
            }
            Err(err) => {
                checkout.mark_failed(err.to_string());
                self.repo.update_checkout(&checkout).await?;
                return Err(err);
            }
        }

        // Best-effort: checkout success should not be rolled back by notification failure.
        if let Err(err) = self
            .notification_sender
            .notify_checkout_created(&checkout)
            .await
        {
            tracing::warn!(checkout_id = %checkout.id, error = %err, "Telegram notification failed");
        }

        self.build_checkout_view(checkout).await
    }

    #[instrument(skip(self))]
    pub async fn get_checkout(&self, checkout_id: Uuid) -> Result<CheckoutView, DomainError> {
        let checkout = self
            .repo
            .find_checkout_by_id(checkout_id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Checkout {}", checkout_id)))?;
        self.build_checkout_view(checkout).await
    }

    #[instrument(skip(self, cmd))]
    pub async fn mark_paid(
        &self,
        checkout_id: Uuid,
        cmd: MarkPaidCommand,
    ) -> Result<(), DomainError> {
        let mut checkout = self
            .repo
            .find_checkout_by_id(checkout_id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Checkout {}", checkout_id)))?;

        checkout.external_payment_id = Some(cmd.provider_payment_id);
        checkout.mark_paid();
        self.repo.update_checkout(&checkout).await
    }

    #[instrument(skip(self, cmd))]
    pub async fn mark_failed(
        &self,
        checkout_id: Uuid,
        cmd: MarkFailedCommand,
    ) -> Result<(), DomainError> {
        let mut checkout = self
            .repo
            .find_checkout_by_id(checkout_id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Checkout {}", checkout_id)))?;

        checkout.mark_failed(cmd.reason);
        self.repo.update_checkout(&checkout).await
    }

    #[instrument(skip(self, cmd))]
    pub async fn handle_provider_webhook(
        &self,
        provider: &str,
        cmd: ProviderWebhookCommand,
    ) -> Result<(), DomainError> {
        cmd.validate()
            .map_err(|e| DomainError::ValidationError(e.to_string()))?;

        let mut checkout = self
            .repo
            .find_checkout_by_id(cmd.checkout_id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Checkout {}", cmd.checkout_id)))?;

        if self
            .repo
            .transaction_exists(provider, &cmd.provider_payment_id)
            .await?
        {
            return Ok(());
        }

        checkout.external_payment_id = Some(cmd.provider_payment_id.clone());

        let normalized_status = cmd.status.to_lowercase();
        let webhook_reason = cmd.reason.clone();
        if normalized_status == "paid"
            || normalized_status == "success"
            || normalized_status == "completed"
        {
            checkout.mark_paid();
        } else {
            checkout.mark_failed(
                webhook_reason.unwrap_or_else(|| {
                    format!("{} webhook status: {}", provider, normalized_status)
                }),
            );
        }

        self.repo.update_checkout(&checkout).await?;

        let tx = PaymentTransaction::create(
            checkout.id,
            provider.to_string(),
            cmd.provider_payment_id,
            checkout.amount_cents,
            checkout.currency,
            normalized_status,
            serde_json::json!({
                "checkout_id": checkout.id,
                "provider": provider,
                "status": cmd.status,
                "reason": cmd.reason,
            }),
        );
        self.repo.create_transaction(&tx).await
    }

    #[instrument(skip(self, payload_bytes))]
    pub async fn handle_provider_webhook_signed(
        &self,
        provider: &str,
        signature: &str,
        payload_bytes: &[u8],
    ) -> Result<(), DomainError> {
        let secret = self.webhook_secret(provider)?;
        verify_hmac_sha256(secret, signature, payload_bytes)?;

        let cmd: ProviderWebhookCommand = serde_json::from_slice(payload_bytes)
            .map_err(|e| DomainError::ValidationError(format!("Invalid webhook payload: {}", e)))?;

        self.handle_provider_webhook(provider, cmd).await
    }

    fn webhook_secret(&self, provider: &str) -> Result<&str, DomainError> {
        let secret = match provider {
            "paypal" => &self.paypal_webhook_secret,
            "stripe" => &self.stripe_webhook_secret,
            "oxapay" => &self.oxapay_webhook_secret,
            _ => {
                return Err(DomainError::ValidationError(format!(
                    "Unsupported webhook provider: {}",
                    provider
                )))
            }
        };

        if secret.is_empty() {
            return Err(DomainError::InfrastructureError(format!(
                "Webhook secret for {} is empty",
                provider
            )));
        }

        Ok(secret)
    }

    async fn build_checkout_view(&self, checkout: Checkout) -> Result<CheckoutView, DomainError> {
        let transactions = self.repo.list_transactions_by_checkout(checkout.id).await?;
        Ok(CheckoutView::from_parts(checkout, transactions))
    }
}

fn verify_hmac_sha256(secret: &str, signature: &str, payload: &[u8]) -> Result<(), DomainError> {
    type HmacSha256 = Hmac<Sha256>;

    let normalized_signature = signature
        .trim()
        .strip_prefix("sha256=")
        .or_else(|| signature.trim().strip_prefix("v1="))
        .unwrap_or(signature.trim());

    let provided = hex::decode(normalized_signature)
        .map_err(|_| DomainError::Unauthorized("Invalid webhook signature format".into()))?;

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|_| DomainError::InfrastructureError("Cannot initialize HMAC".into()))?;
    mac.update(payload);

    mac.verify_slice(&provided)
        .map_err(|_| DomainError::Unauthorized("Webhook signature verification failed".into()))
}

fn parse_payment_method(input: &str) -> Result<PaymentMethod, DomainError> {
    match input.to_lowercase().as_str() {
        "paypal" => Ok(PaymentMethod::Paypal),
        "stripe" | "strip" => Ok(PaymentMethod::Stripe),
        "oxapay" | "crypto" => Ok(PaymentMethod::OxaPay),
        _ => Err(DomainError::ValidationError(
            "payment_method must be paypal, stripe, or oxapay".into(),
        )),
    }
}
