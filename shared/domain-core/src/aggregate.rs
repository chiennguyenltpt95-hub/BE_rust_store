use crate::domain_event::DomainEvent;
use crate::entity::Entity;

/// Trait AggregateRoot — Aggregate là đơn vị nhất quán trong DDD.
/// Mọi thay đổi state phải đi qua aggregate root.
/// Aggregate root giữ danh sách domain events chưa được dispatch.
pub trait AggregateRoot: Entity {
    fn uncommitted_events(&self) -> &Vec<Box<dyn DomainEvent>>;
    fn mark_events_as_committed(&mut self);
    fn version(&self) -> u64;
}

/// Base struct hỗ trợ implement AggregateRoot
#[derive(Debug, Default)]
pub struct AggregateBase {
    uncommitted_events: Vec<Box<dyn DomainEvent>>,
    version: u64,
}

impl AggregateBase {
    pub fn new() -> Self {
        Self {
            uncommitted_events: Vec::new(),
            version: 0,
        }
    }

    pub fn record_event(&mut self, event: Box<dyn DomainEvent>) {
        self.uncommitted_events.push(event);
    }

    pub fn uncommitted_events(&self) -> &Vec<Box<dyn DomainEvent>> {
        &self.uncommitted_events
    }

    pub fn mark_committed(&mut self) {
        self.uncommitted_events.clear();
        self.version += 1;
    }

    pub fn version(&self) -> u64 {
        self.version
    }
}
