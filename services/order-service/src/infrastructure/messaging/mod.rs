use anyhow::Result;
use prost::Message;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;

use crate::application::ports::MailMessage;

pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/store.events.rs"));
}

pub struct KafkaEventPublisher {
    producer: FutureProducer,
    topic: String,
}

impl KafkaEventPublisher {
    pub fn new(brokers: &str, topic: &str) -> Result<Self> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .set("acks", "all")
            .create()
            .map_err(|e| anyhow::anyhow!("Failed to create Kafka producer: {}", e))?;

        Ok(Self {
            producer,
            topic: topic.to_string(),
        })
    }

    pub async fn publish_order_confirmed_mail(
        &self,
        aggregate_id: &str,
        mail: &MailMessage,
    ) -> Result<(), domain_core::error::DomainError> {
        let event = proto::OrderConfirmedMailEvent {
            to: mail.to.clone(),
            to_name: mail.to_name.clone().unwrap_or_default(),
            subject: mail.subject.clone(),
            template_name: mail.template_name.clone(),
            context_json: mail.context.to_string(),
        };

        let envelope = proto::DomainEventEnvelope {
            aggregate_id: aggregate_id.to_string(),
            event_type: "order.confirmed.mail".to_string(),
            occurred_at: chrono::Utc::now().to_rfc3339(),
            payload: Some(proto::domain_event_envelope::Payload::OrderConfirmedMail(
                event,
            )),
        };

        let bytes = envelope.encode_to_vec();

        self.producer
            .send(
                FutureRecord::to(&self.topic)
                    .key(aggregate_id)
                    .payload(&bytes),
                Duration::from_secs(5),
            )
            .await
            .map_err(|(e, _)| {
                domain_core::error::DomainError::InfrastructureError(format!(
                    "Kafka publish error: {}",
                    e
                ))
            })?;

        Ok(())
    }
}
