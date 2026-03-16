use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Deserialize, Serialize};

/// Trait DomainEvent — base cho mọi domain event.
pub trait DomainEvent: std::fmt::Debug + Send + Sync {
    fn event_type(&self) -> &str;
    fn aggregate_id(&self) -> Uuid;
    fn occurred_at(&self) -> DateTime<Utc>;
}

/// Envelope bọc domain event kèm metadata để publish lên message bus.
#[derive(Debug, Serialize, Deserialize)]
pub struct DomainEventEnvelope {
    pub event_id: Uuid,
    pub event_type: String,
    pub aggregate_id: Uuid,
    pub aggregate_type: String,
    pub payload: serde_json::Value,
    pub occurred_at: DateTime<Utc>,
    pub version: u64,
}

impl DomainEventEnvelope {
    pub fn new(
        event_type: impl Into<String>,
        aggregate_id: Uuid,
        aggregate_type: impl Into<String>,
        payload: serde_json::Value,
        version: u64,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            event_type: event_type.into(),
            aggregate_id,
            aggregate_type: aggregate_type.into(),
            payload,
            occurred_at: Utc::now(),
            version,
        }
    }
}
