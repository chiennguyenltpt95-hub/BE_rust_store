use async_trait::async_trait;
use domain_core::error::DomainError;

use crate::application::ports::notification_sender::NotificationSenderPort;
use crate::domain::entities::checkout::Checkout;

pub struct NotificationServiceClient {
    http: reqwest::Client,
    base_url: String,
    telegram_recipient: String,
}

impl NotificationServiceClient {
    pub fn new(base_url: String, telegram_recipient: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            telegram_recipient,
        }
    }
}

#[async_trait]
impl NotificationSenderPort for NotificationServiceClient {
    async fn notify_checkout_created(&self, checkout: &Checkout) -> Result<(), DomainError> {
        if self.telegram_recipient.trim().is_empty() {
            return Err(DomainError::InfrastructureError(
                "TELEGRAM_CHAT_ID is empty".into(),
            ));
        }

        let payload = serde_json::json!({
            "checkout_id": checkout.id,
            "user_id": checkout.user_id,
            "cart_id": checkout.cart_id,
            "amount_cents": checkout.amount_cents,
            "currency": checkout.currency,
            "checkout_url": checkout.checkout_url,
            "text": format!(
                "New checkout created\\ncheckout_id: {}\\namount: {} {}\\nurl: {}",
                checkout.id,
                checkout.amount_cents,
                checkout.currency,
                checkout.checkout_url.clone().unwrap_or_default(),
            )
        });

        let body = serde_json::json!({
            "channel": "telegram",
            "recipient": self.telegram_recipient,
            "template_name": "checkout_created",
            "payload": payload,
            "max_attempts": 5
        });

        let resp = self
            .http
            .post(format!("{}/api/v1/notifications/send", self.base_url))
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                DomainError::InfrastructureError(format!("Notification request failed: {}", e))
            })?;

        if !resp.status().is_success() {
            let text = resp
                .text()
                .await
                .unwrap_or_else(|_| "<empty body>".to_string());
            return Err(DomainError::InfrastructureError(format!(
                "Notification service returned non-success: {}",
                text
            )));
        }

        Ok(())
    }
}
