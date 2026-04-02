use chrono::{DateTime, Utc};
use domain_core::error::DomainError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Category {
    pub fn create(id: Uuid, name: String, slug: String) -> Result<Self, DomainError> {
        if name.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Category name cannot be empty".into(),
            ));
        }
        if slug.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Category slug cannot be empty".into(),
            ));
        }

        let now = Utc::now();
        Ok(Self {
            id,
            name,
            slug,
            created_at: now,
            updated_at: now,
        })
    }
}
