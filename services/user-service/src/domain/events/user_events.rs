use chrono::{DateTime, Utc};
use domain_core::domain_event::DomainEvent;
use uuid::Uuid;

#[derive(Debug, serde::Serialize)]
pub struct UserCreated {
    pub user_id: Uuid,
    pub email: String,
    pub full_name: String,
    pub role: String,
    pub occurred_at: DateTime<Utc>,
}

impl DomainEvent for UserCreated {
    fn event_type(&self) -> &str { "user.created" }
    fn aggregate_id(&self) -> Uuid { self.user_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
}

#[derive(Debug, serde::Serialize)]
pub struct UserUpdated {
    pub user_id: Uuid,
    pub full_name: String,
    pub occurred_at: DateTime<Utc>,
}

impl DomainEvent for UserUpdated {
    fn event_type(&self) -> &str { "user.updated" }
    fn aggregate_id(&self) -> Uuid { self.user_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
}

#[derive(Debug, serde::Serialize)]
pub struct UserDeleted {
    pub user_id: Uuid,
    pub occurred_at: DateTime<Utc>,
}

impl DomainEvent for UserDeleted {
    fn event_type(&self) -> &str { "user.deleted" }
    fn aggregate_id(&self) -> Uuid { self.user_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
}
