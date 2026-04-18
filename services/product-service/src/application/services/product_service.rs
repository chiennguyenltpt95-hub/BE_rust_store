use domain_core::error::DomainError;
use serde_json::json;
use std::sync::Arc;
use tracing::instrument;
use uuid::Uuid;
use validator::Validate;

use crate::application::commands::{CreateProductCommand, UpdateProductCommand};
use crate::application::queries::{ListProductsQuery, ProductView};
use crate::domain::entities::Product;
use crate::domain::repositories::ProductRepository;

pub struct ProductAppService {
    product_repo: Arc<dyn ProductRepository>,
}

impl ProductAppService {
    pub fn new(product_repo: Arc<dyn ProductRepository>) -> Self {
        Self { product_repo }
    }

    #[instrument(skip(self, cmd))]
    pub async fn create_product(&self, cmd: CreateProductCommand) -> Result<Uuid, DomainError> {
        cmd.validate()
            .map_err(|e| DomainError::ValidationError(e.to_string()))?;

        self.validate_category_assignment(cmd.category_id).await?;

        let product = Product::create(
            Uuid::new_v4(),
            cmd.name,
            cmd.description,
            cmd.sku,
            cmd.category_id,
            cmd.supplier_id,
            cmd.supplier_url,
            cmd.product_type,
            cmd.attributes.unwrap_or_else(|| json!({})),
            cmd.image_urls,
            cmd.price_cents,
            cmd.cost_price_cents,
            cmd.sale_price_cents,
            cmd.sizes,
            cmd.colors,
            cmd.stock,
            cmd.weight_grams,
            cmd.status,
            cmd.tags,
        )?;

        let product_id = product.id;
        self.product_repo.save(&product).await?;

        Ok(product_id)
    }

    #[instrument(skip(self))]
    pub async fn get_product(&self, id: Uuid) -> Result<ProductView, DomainError> {
        let product = self
            .product_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Product {}", id)))?;

        Ok(product.into())
    }

    #[instrument(skip(self, cmd))]
    pub async fn update_product(
        &self,
        id: Uuid,
        cmd: UpdateProductCommand,
    ) -> Result<(), DomainError> {
        cmd.validate()
            .map_err(|e| DomainError::ValidationError(e.to_string()))?;

        self.validate_category_assignment(cmd.category_id).await?;

        let mut product = self
            .product_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Product {}", id)))?;

        let attributes = cmd.attributes.unwrap_or_else(|| product.attributes.clone());

        product.update(
            cmd.name,
            cmd.description,
            cmd.sku,
            cmd.category_id,
            cmd.supplier_id,
            cmd.supplier_url,
            cmd.product_type,
            attributes,
            cmd.image_urls,
            cmd.price_cents,
            cmd.cost_price_cents,
            cmd.sale_price_cents,
            cmd.sizes,
            cmd.colors,
            cmd.stock,
            cmd.weight_grams,
            cmd.status,
            cmd.tags,
        )?;
        self.product_repo.update(&product).await
    }

    #[instrument(skip(self))]
    pub async fn delete_product(&self, id: Uuid) -> Result<(), DomainError> {
        let mut product = self
            .product_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Product {}", id)))?;

        product.deactivate();
        self.product_repo.update(&product).await?;
        self.product_repo.delete(id).await
    }

    #[instrument(skip(self, query))]
    pub async fn list_products(
        &self,
        query: ListProductsQuery,
    ) -> Result<Vec<ProductView>, DomainError> {
        let all = self.product_repo.find_all().await?;

        let search = query.search.map(|s| s.to_lowercase());

        let filtered = all
            .into_iter()
            .filter(|product| {
                if let Some(s) = &search {
                    let in_name = product.name.to_lowercase().contains(s);
                    let in_desc = product
                        .description
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(s))
                        .unwrap_or(false);
                    if !in_name && !in_desc {
                        return false;
                    }
                }

                if let Some(min_price) = query.min_price_cents {
                    if product.price_cents < min_price {
                        return false;
                    }
                }

                if let Some(category_id) = query.category_id {
                    if product.category_id != Some(category_id) {
                        return false;
                    }
                }

                if let Some(sku) = &query.sku {
                    let sku_matches = product
                        .sku
                        .as_ref()
                        .map(|product_sku| product_sku.eq_ignore_ascii_case(sku))
                        .unwrap_or(false);
                    if !sku_matches {
                        return false;
                    }
                }

                if let Some(product_type) = &query.product_type {
                    let type_matches = product
                        .product_type
                        .as_ref()
                        .map(|kind| kind.eq_ignore_ascii_case(product_type))
                        .unwrap_or(false);
                    if !type_matches {
                        return false;
                    }
                }

                if let Some(category_slug) = &query.category_slug {
                    let slug_matches = product
                        .category_slug
                        .as_ref()
                        .map(|slug| slug.eq_ignore_ascii_case(category_slug))
                        .unwrap_or(false);
                    if !slug_matches {
                        return false;
                    }
                }

                if let Some(category_active) = query.category_active {
                    if product.category_is_active != Some(category_active) {
                        return false;
                    }
                }

                if let Some(target_gender) = &query.target_gender {
                    let gender_matches = product
                        .category_target_gender
                        .as_ref()
                        .map(|gender| gender.eq_ignore_ascii_case(target_gender))
                        .unwrap_or(false);
                    if !gender_matches {
                        return false;
                    }
                }

                if let Some(max_price) = query.max_price_cents {
                    if product.price_cents > max_price {
                        return false;
                    }
                }

                if query.in_stock.unwrap_or(false) && product.stock <= 0 {
                    return false;
                }

                true
            })
            .map(ProductView::from)
            .collect();

        Ok(filtered)
    }

    async fn validate_category_assignment(
        &self,
        category_id: Option<Uuid>,
    ) -> Result<(), DomainError> {
        let Some(category_id) = category_id else {
            return Ok(());
        };

        let category = self
            .product_repo
            .find_category_by_id(category_id)
            .await?
            .ok_or_else(|| {
                DomainError::ValidationError(format!("Category {} does not exist", category_id))
            })?;

        if !category.is_active {
            return Err(DomainError::ValidationError(format!(
                "Category {} is inactive",
                category_id
            )));
        }

        Ok(())
    }
}
