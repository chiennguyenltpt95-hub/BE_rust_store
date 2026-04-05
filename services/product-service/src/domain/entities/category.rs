use chrono::{DateTime, Utc};
use domain_core::error::DomainError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub parent_id: Option<Uuid>,
    pub display_order: i32,
    pub is_active: bool,
    pub target_gender: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Category {
    pub fn create(
        id: Uuid,
        name: String,
        slug: String,
        description: Option<String>,
        image_url: Option<String>,
        parent_id: Option<Uuid>,
        display_order: i32,
        is_active: bool,
        target_gender: String,
    ) -> Result<Self, DomainError> {
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
        if display_order < 0 {
            return Err(DomainError::ValidationError(
                "Category display_order cannot be negative".into(),
            ));
        }

        let normalized_gender = target_gender.trim().to_lowercase();
        if normalized_gender != "female" && normalized_gender != "unisex" {
            return Err(DomainError::ValidationError(
                "Category target_gender must be female or unisex".into(),
            ));
        }

        let now = Utc::now();
        Ok(Self {
            id,
            name,
            slug,
            description,
            image_url,
            parent_id,
            display_order,
            is_active,
            target_gender: normalized_gender,
            created_at: now,
            updated_at: now,
        })
    }
}
