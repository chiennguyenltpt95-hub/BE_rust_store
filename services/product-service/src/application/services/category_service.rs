use domain_core::error::DomainError;
use std::sync::Arc;
use tracing::instrument;
use uuid::Uuid;
use validator::Validate;

use crate::application::commands::CreateCategoryCommand;
use crate::application::queries::CategoryView;
use crate::domain::entities::Category;
use crate::domain::repositories::ProductRepository;

pub struct CategoryAppService {
    product_repo: Arc<dyn ProductRepository>,
}

impl CategoryAppService {
    pub fn new(product_repo: Arc<dyn ProductRepository>) -> Self {
        Self { product_repo }
    }

    #[instrument(skip(self, cmd))]
    pub async fn create_category(&self, cmd: CreateCategoryCommand) -> Result<Uuid, DomainError> {
        cmd.validate()
            .map_err(|e| DomainError::ValidationError(e.to_string()))?;

        let slug = cmd.slug.unwrap_or_else(|| slugify(&cmd.name));

        if self
            .product_repo
            .find_category_by_slug(&slug)
            .await?
            .is_some()
        {
            return Err(DomainError::Conflict(format!(
                "Category slug '{}' already exists",
                slug
            )));
        }

        let category = Category::create(Uuid::new_v4(), cmd.name, slug)?;
        let id = category.id;
        self.product_repo.save_category(&category).await?;
        Ok(id)
    }

    #[instrument(skip(self))]
    pub async fn get_category(&self, id: Uuid) -> Result<CategoryView, DomainError> {
        let category = self
            .product_repo
            .find_category_by_id(id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Category {}", id)))?;

        Ok(category.into())
    }

    #[instrument(skip(self))]
    pub async fn list_categories(&self) -> Result<Vec<CategoryView>, DomainError> {
        let categories = self.product_repo.list_categories().await?;
        Ok(categories.into_iter().map(CategoryView::from).collect())
    }

    #[instrument(skip(self))]
    pub async fn delete_category(&self, id: Uuid) -> Result<(), DomainError> {
        self.product_repo.delete_category(id).await
    }
}

fn slugify(input: &str) -> String {
    let lowered = input.trim().to_lowercase();
    let mut out = String::with_capacity(lowered.len());
    let mut last_dash = false;

    for ch in lowered.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }

    out.trim_matches('-').to_string()
}
