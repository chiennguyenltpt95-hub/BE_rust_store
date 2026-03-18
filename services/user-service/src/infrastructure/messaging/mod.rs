use anyhow::Result;
use async_trait::async_trait;
use domain_core::domain_event::DomainEvent;
use domain_core::error::DomainError;
use prost::Message;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;
use tracing::{info, warn};

/// Generated protobuf types
pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/store.events.rs"));
}

/// Port: EventPublisher
#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish(&self, event: &dyn DomainEvent) -> Result<(), DomainError>;
}

/// Kafka implementation — Protobuf encoding
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

        info!("Kafka producer connected to {}", brokers);
        Ok(Self {
            producer,
            topic: topic.to_string(),
        })
    }
}

#[async_trait]
impl EventPublisher for KafkaEventPublisher {
    async fn publish(&self, event: &dyn DomainEvent) -> Result<(), DomainError> {
        // Build protobuf payload từ DomainEvent
        let payload_json = event.payload();
        let proto_payload = build_proto_payload(event, &payload_json);

        let envelope = proto::DomainEventEnvelope {
            aggregate_id: event.aggregate_id().to_string(),
            event_type: event.event_type().to_string(),
            occurred_at: event.occurred_at().to_rfc3339(),
            payload: Some(proto_payload),
        };

        // Encode sang protobuf binary
        let bytes = envelope.encode_to_vec();

        // Dùng aggregate_id làm key → đảm bảo thứ tự events cùng aggregate
        let key = event.aggregate_id().to_string();

        let record = FutureRecord::to(&self.topic)
            .key(&key)
            .payload(&bytes);

        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(e, _)| DomainError::InfrastructureError(format!("Kafka publish error: {}", e)))?;

        Ok(())
    }
}

/// Map DomainEvent → protobuf oneof payload
fn build_proto_payload(
    event: &dyn DomainEvent,
    json: &serde_json::Value,
) -> proto::domain_event_envelope::Payload {
    match event.event_type() {
        "user.created" => {
            proto::domain_event_envelope::Payload::UserCreated(proto::UserCreatedEvent {
                user_id: json_str(json, "user_id"),
                email: json_str(json, "email"),
                full_name: json_str(json, "full_name"),
                role: json_str(json, "role"),
            })
        }
        "user.updated" => {
            proto::domain_event_envelope::Payload::UserUpdated(proto::UserUpdatedEvent {
                user_id: json_str(json, "user_id"),
                full_name: json_str(json, "full_name"),
            })
        }
        "user.deleted" => {
            proto::domain_event_envelope::Payload::UserDeleted(proto::UserDeletedEvent {
                user_id: json_str(json, "user_id"),
            })
        }
        _ => {
            proto::domain_event_envelope::Payload::UserCreated(proto::UserCreatedEvent::default())
        }
    }
}

fn json_str(val: &serde_json::Value, key: &str) -> String {
    val.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string()
}

/// Dùng trong dev/test — không publish thật
pub struct NoopEventPublisher;

#[async_trait]
impl EventPublisher for NoopEventPublisher {
    async fn publish(&self, event: &dyn DomainEvent) -> Result<(), DomainError> {
        warn!(event_type = %event.event_type(), "NoopEventPublisher: event discarded");
        Ok(())
    }
}
