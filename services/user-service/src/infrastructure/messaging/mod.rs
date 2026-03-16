use async_trait::async_trait;
use async_nats::Client;
use domain_core::domain_event::DomainEvent;
use domain_core::error::DomainError;
use serde_json::json;
use tracing::warn;
use anyhow::Result;

/// Port: EventPublisher
#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish(&self, event: &dyn DomainEvent) -> Result<(), DomainError>;
}

/// NATS implementation
pub struct NatsEventPublisher {
    client: Client,
}

impl NatsEventPublisher {
    pub fn new(client: Client) -> Self {
        Self { client }
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

        self.client
            .publish(subject, bytes.into())
            .await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(())
    }
}

pub async fn connect_nats(url: &str) -> Result<Client> {
    let client = async_nats::connect(url).await?;
    Ok(client)
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
