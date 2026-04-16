use async_trait::async_trait;
use domain_core::error::DomainError;
use uuid::Uuid;

use crate::domain::entities::{Category, Product};

#[async_trait]
pub trait ProductRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Product>, DomainError>;
    async fn save(&self, product: &Product) -> Result<(), DomainError>;
    async fn update(&self, product: &Product) -> Result<(), DomainError>;
    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;
    async fn find_all(&self) -> Result<Vec<Product>, DomainError>;
    async fn save_category(&self, category: &Category) -> Result<(), DomainError>;
    async fn find_category_by_id(&self, id: Uuid) -> Result<Option<Category>, DomainError>;
    async fn find_category_by_slug(&self, slug: &str) -> Result<Option<Category>, DomainError>;
    async fn list_categories(&self) -> Result<Vec<Category>, DomainError>;
    async fn delete_category(&self, id: Uuid) -> Result<(), DomainError>;
    async fn search_categories(&self, query: &str) -> Result<Vec<Category>, DomainError>;
}
