use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Trait đánh dấu một Entity trong DDD.
/// Entity có identity (ID) duy nhất và vòng đời riêng.
pub trait Entity {
    type Id: Clone + Eq + std::hash::Hash + std::fmt::Debug;

    fn id(&self) -> &Self::Id;
    fn created_at(&self) -> DateTime<Utc>;
    fn updated_at(&self) -> DateTime<Utc>;
}

/// EntityId wrapper cho Uuid
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct EntityId(pub Uuid);

impl EntityId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(id: Uuid) -> Self {
        Self(id)
    }

    pub fn value(&self) -> Uuid {
        self.0
    }
}

impl Default for EntityId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
