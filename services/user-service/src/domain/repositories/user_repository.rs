use async_trait::async_trait;
use domain_core::error::DomainError;
use uuid::Uuid;

use crate::domain::entities::User;
use crate::domain::value_objects::Email;

/// Port (interface) — định nghĩa contract cho User repository.
/// Infrastructure sẽ implement trait này.
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError>;
    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, DomainError>;
    async fn save(&self, user: &User) -> Result<(), DomainError>;
    async fn update(&self, user: &User) -> Result<(), DomainError>;
    async fn set_verified(&self, id: Uuid) -> Result<(), DomainError>;
    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;
    async fn exists_by_email(&self, email: &Email) -> Result<bool, DomainError>;
}
