use async_trait::async_trait;
use crate::entity::Entity;
use crate::error::DomainError;

/// Repository cho Write (Command side)
#[async_trait]
pub trait Repository<T: Entity>: Send + Sync {
    async fn find_by_id(&self, id: &T::Id) -> Result<Option<T>, DomainError>;
    async fn save(&self, entity: &T) -> Result<(), DomainError>;
    async fn delete(&self, id: &T::Id) -> Result<(), DomainError>;
}

/// Repository cho Read (Query side — CQRS)
#[async_trait]
pub trait ReadRepository<T, Filter>: Send + Sync {
    async fn find_all(&self, filter: Filter) -> Result<Vec<T>, DomainError>;
    async fn find_by_id(&self, id: uuid::Uuid) -> Result<Option<T>, DomainError>;
    async fn count(&self, filter: &Filter) -> Result<u64, DomainError>;
}
