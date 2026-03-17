use anyhow::Result;
use async_nats::jetstream::{self, Context as JetStreamContext};
use async_trait::async_trait;
use domain_core::domain_event::DomainEvent;
use domain_core::error::DomainError;
use serde_json::json;
use tracing::{info, warn};

/// Port: EventPublisher
#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish(&self, event: &dyn DomainEvent) -> Result<(), DomainError>;
}

/// NATS JetStream implementation
pub struct NatsEventPublisher {
    js: JetStreamContext,
}

impl NatsEventPublisher {
    pub fn new(js: JetStreamContext) -> Self {
        Self { js }
    }
}

#[async_trait]
impl EventPublisher for NatsEventPublisher {
    async fn publish(&self, event: &dyn DomainEvent) -> Result<(), DomainError> {
        let subject = format!("events.{}", event.event_type());
        let payload = json!({
            "aggregate_id": event.aggregate_id(),
            "event_type": event.event_type(),
            "occurred_at": event.occurred_at(),
        });

        let bytes = serde_json::to_vec(&payload)
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        // JetStream publish + chờ ACK đảm bảo message không mất
        self.js
            .publish(subject, bytes.into())
            .await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?
            .await // chờ server ACK
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(())
    }
}

pub async fn connect_nats(url: &str) -> Result<async_nats::Client> {
    let client = async_nats::connect(url).await?;
    Ok(client)
}

/// Khởi tạo JetStream context và tạo stream EVENTS
pub async fn create_jetstream(client: &async_nats::Client) -> Result<JetStreamContext> {
    let js = jetstream::new(client.clone());

    // Tạo stream lưu toàn bộ events.* nếu chưa có
    js.get_or_create_stream(jetstream::stream::Config {
        name: "EVENTS".to_string(),
        subjects: vec!["events.>".to_string()],
        retention: jetstream::stream::RetentionPolicy::Limits,
        storage: jetstream::stream::StorageType::File,
        max_messages: 10_000_000,
        ..Default::default()
    })
    .await
    .map_err(|e| anyhow::anyhow!("Failed to create JetStream EVENTS stream: {}", e))?;

    info!("JetStream stream EVENTS ready");
    Ok(js)
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
