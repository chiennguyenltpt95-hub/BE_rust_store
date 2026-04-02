use async_trait::async_trait;
use domain_core::error::DomainError;
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::domain::entities::{Category, Product};
use crate::domain::repositories::ProductRepository;

pub struct InMemoryProductRepository {
    storage: RwLock<HashMap<Uuid, Product>>,
    categories: RwLock<HashMap<Uuid, Category>>,
}

impl InMemoryProductRepository {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            storage: RwLock::new(HashMap::new()),
            categories: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl ProductRepository for InMemoryProductRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Product>, DomainError> {
        let map = self.storage.read().await;
        Ok(map.get(&id).cloned())
    }

    async fn save(&self, product: &Product) -> Result<(), DomainError> {
        let mut map = self.storage.write().await;
        map.insert(product.id, product.clone());
        Ok(())
    }

    async fn update(&self, product: &Product) -> Result<(), DomainError> {
        let mut map = self.storage.write().await;
        if !map.contains_key(&product.id) {
            return Err(DomainError::NotFound(format!("Product {}", product.id)));
        }
        map.insert(product.id, product.clone());
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        let mut map = self.storage.write().await;
        map.remove(&id);
        Ok(())
    }

    async fn find_all(&self) -> Result<Vec<Product>, DomainError> {
        let map = self.storage.read().await;
        Ok(map.values().cloned().collect())
    }

    async fn save_category(&self, category: &Category) -> Result<(), DomainError> {
        let mut map = self.categories.write().await;
        map.insert(category.id, category.clone());
        Ok(())
    }

    async fn find_category_by_id(&self, id: Uuid) -> Result<Option<Category>, DomainError> {
        let map = self.categories.read().await;
        Ok(map.get(&id).cloned())
    }

    async fn find_category_by_slug(&self, slug: &str) -> Result<Option<Category>, DomainError> {
        let map = self.categories.read().await;
        Ok(map.values().find(|c| c.slug == slug).cloned())
    }

    async fn list_categories(&self) -> Result<Vec<Category>, DomainError> {
        let map = self.categories.read().await;
        Ok(map.values().cloned().collect())
    }

    async fn delete_category(&self, id: Uuid) -> Result<(), DomainError> {
        let mut map = self.categories.write().await;
        map.remove(&id);
        Ok(())
    }
}
