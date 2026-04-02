use anyhow::Result;
use async_trait::async_trait;
use domain_core::error::DomainError;
use prost::Message;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;

use crate::application::ports::NotificationSenderPort;
use crate::domain::entities::checkout::Checkout;

pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/store.events.rs"));
}

pub struct KafkaNotificationPublisher {
    producer: FutureProducer,
    topic: String,
    telegram_recipient: String,
}

impl KafkaNotificationPublisher {
    pub fn new(brokers: &str, topic: &str, telegram_recipient: String) -> Result<Self> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .set("acks", "all")
            .create()
            .map_err(|e| anyhow::anyhow!("Failed to create Kafka producer: {}", e))?;

        Ok(Self {
            producer,
            topic: topic.to_string(),
            telegram_recipient,
        })
    }
}

#[async_trait]
impl NotificationSenderPort for KafkaNotificationPublisher {
    async fn notify_checkout_created(&self, checkout: &Checkout) -> Result<(), DomainError> {
        if self.telegram_recipient.trim().is_empty() {
            return Err(DomainError::InfrastructureError(
                "TELEGRAM_CHAT_ID is empty".into(),
            ));
        }

        let payload_json = serde_json::json!({
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

        let event = proto::CheckoutCreatedNotificationEvent {
            checkout_id: checkout.id.to_string(),
            user_id: checkout.user_id.to_string(),
            cart_id: checkout.cart_id.to_string(),
            amount_cents: checkout.amount_cents,
            currency: checkout.currency.clone(),
            checkout_url: checkout.checkout_url.clone().unwrap_or_default(),
            channel: "telegram".into(),
            recipient: self.telegram_recipient.clone(),
            template_name: "checkout_created".into(),
            payload_json: payload_json.to_string(),
            max_attempts: 5,
        };

        let envelope = proto::DomainEventEnvelope {
            aggregate_id: checkout.id.to_string(),
            event_type: "checkout.created.notification".into(),
            occurred_at: chrono::Utc::now().to_rfc3339(),
            payload: Some(proto::domain_event_envelope::Payload::CheckoutCreatedNotification(event)),
        };

        let bytes = envelope.encode_to_vec();

        self.producer
            .send(
                FutureRecord::to(&self.topic)
                    .key(&checkout.id.to_string())
                    .payload(&bytes),
                Duration::from_secs(5),
            )
            .await
            .map_err(|(e, _)| {
                DomainError::InfrastructureError(format!("Kafka publish error: {}", e))
            })?;

        Ok(())
    }
}
