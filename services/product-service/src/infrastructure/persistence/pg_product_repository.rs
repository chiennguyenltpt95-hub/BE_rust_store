use async_trait::async_trait;
use domain_core::error::DomainError;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::product::ProductStatus;
use crate::domain::entities::{Category, Product};
use crate::domain::repositories::ProductRepository;

pub struct PgProductRepository {
    pool: PgPool,
}

impl PgProductRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct ProductRow {
    id: Uuid,
    name: String,
    description: Option<String>,
    sku: Option<String>,
    category_id: Option<Uuid>,
    category_name: Option<String>,
    category_slug: Option<String>,
    category_is_active: Option<bool>,
    category_target_gender: Option<String>,
    product_type: Option<String>,
    attributes: Value,
    image_urls: Vec<String>,
    price_cents: i64,
    stock: i32,
    status: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(sqlx::FromRow)]
struct CategoryRow {
    id: Uuid,
    name: String,
    slug: String,
    description: Option<String>,
    image_url: Option<String>,
    parent_id: Option<Uuid>,
    display_order: i32,
    is_active: bool,
    target_gender: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl CategoryRow {
    fn into_category(self) -> Category {
        Category {
            id: self.id,
            name: self.name,
            slug: self.slug,
            description: self.description,
            image_url: self.image_url,
            parent_id: self.parent_id,
            display_order: self.display_order,
            is_active: self.is_active,
            target_gender: self.target_gender,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl ProductRow {
    fn into_product(self) -> Product {
        let status = match self.status.as_str() {
            "draft" => ProductStatus::Draft,
            "inactive" => ProductStatus::Inactive,
            _ => ProductStatus::Active,
        };

        Product {
            id: self.id,
            name: self.name,
            description: self.description,
            sku: self.sku,
            category_id: self.category_id,
            category_name: self.category_name,
            category_slug: self.category_slug,
            category_is_active: self.category_is_active,
            category_target_gender: self.category_target_gender,
            product_type: self.product_type,
            attributes: self.attributes,
            image_urls: self.image_urls,
            price_cents: self.price_cents,
            stock: self.stock,
            status,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[async_trait]
impl ProductRepository for PgProductRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Product>, DomainError> {
        let row: Option<ProductRow> = sqlx::query_as(
            r#"SELECT p.id, p.name, p.description, p.sku, p.category_id, c.name AS category_name,
                    c.slug AS category_slug, c.is_active AS category_is_active,
                    c.target_gender AS category_target_gender,
                    p.product_type, p.attributes, p.image_urls,
                 p.price_cents, p.stock, p.status, p.created_at, p.updated_at
             FROM products p
             LEFT JOIN categories c ON c.id = p.category_id
             WHERE p.id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(row.map(ProductRow::into_product))
    }

    async fn save(&self, product: &Product) -> Result<(), DomainError> {
        let status = format!("{:?}", product.status).to_lowercase();

        sqlx::query(
            r#"INSERT INTO products
               (id, name, description, sku, category_id, product_type, attributes, image_urls, price_cents, stock, status, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)"#,
        )
        .bind(product.id)
        .bind(&product.name)
        .bind(&product.description)
           .bind(&product.sku)
           .bind(product.category_id)
           .bind(&product.product_type)
           .bind(&product.attributes)
           .bind(&product.image_urls)
           .bind(product.price_cents)
           .bind(product.stock)
           .bind(status)
           .bind(product.created_at)
           .bind(product.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(())
    }

    async fn update(&self, product: &Product) -> Result<(), DomainError> {
        let status = format!("{:?}", product.status).to_lowercase();

        let res = sqlx::query(
            r#"UPDATE products
               SET name = $2,
                   description = $3,
                   sku = $4,
                   category_id = $5,
                   product_type = $6,
                   attributes = $7,
                   image_urls = $8,
                   price_cents = $9,
                   stock = $10,
                   status = $11,
                   updated_at = $12
               WHERE id = $1"#,
        )
        .bind(product.id)
        .bind(&product.name)
        .bind(&product.description)
        .bind(&product.sku)
        .bind(product.category_id)
        .bind(&product.product_type)
        .bind(&product.attributes)
        .bind(&product.image_urls)
        .bind(product.price_cents)
        .bind(product.stock)
        .bind(status)
        .bind(product.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        if res.rows_affected() == 0 {
            return Err(DomainError::NotFound(format!("Product {}", product.id)));
        }

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM products WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(())
    }

    async fn find_all(&self) -> Result<Vec<Product>, DomainError> {
        let rows: Vec<ProductRow> = sqlx::query_as(
            r#"SELECT p.id, p.name, p.description, p.sku, p.category_id, c.name AS category_name,
                    c.slug AS category_slug, c.is_active AS category_is_active,
                    c.target_gender AS category_target_gender,
                    p.product_type, p.attributes, p.image_urls,
                 p.price_cents, p.stock, p.status, p.created_at, p.updated_at
             FROM products p
             LEFT JOIN categories c ON c.id = p.category_id"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(rows.into_iter().map(ProductRow::into_product).collect())
    }

    async fn save_category(&self, category: &Category) -> Result<(), DomainError> {
        sqlx::query(
            r#"INSERT INTO categories
               (id, name, slug, description, image_url, parent_id, display_order, is_active, target_gender, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"#,
        )
        .bind(category.id)
        .bind(&category.name)
        .bind(&category.slug)
        .bind(&category.description)
        .bind(&category.image_url)
        .bind(category.parent_id)
        .bind(category.display_order)
        .bind(category.is_active)
        .bind(&category.target_gender)
        .bind(category.created_at)
        .bind(category.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(())
    }

    async fn find_category_by_id(&self, id: Uuid) -> Result<Option<Category>, DomainError> {
        let row: Option<CategoryRow> = sqlx::query_as(
            r#"SELECT id, name, slug, description, image_url, parent_id, display_order,
                      is_active, target_gender, created_at, updated_at
               FROM categories
               WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(row.map(CategoryRow::into_category))
    }

    async fn find_category_by_slug(&self, slug: &str) -> Result<Option<Category>, DomainError> {
        let row: Option<CategoryRow> = sqlx::query_as(
            r#"SELECT id, name, slug, description, image_url, parent_id, display_order,
                      is_active, target_gender, created_at, updated_at
               FROM categories
               WHERE slug = $1"#,
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(row.map(CategoryRow::into_category))
    }

    async fn list_categories(&self) -> Result<Vec<Category>, DomainError> {
        let rows: Vec<CategoryRow> = sqlx::query_as(
             r#"SELECT id, name, slug, description, image_url, parent_id, display_order,
                 is_active, target_gender, created_at, updated_at
               FROM categories
             ORDER BY display_order ASC, name ASC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(rows.into_iter().map(CategoryRow::into_category).collect())
    }

    async fn delete_category(&self, id: Uuid) -> Result<(), DomainError> {
        let res = sqlx::query("DELETE FROM categories WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        if res.rows_affected() == 0 {
            return Err(DomainError::NotFound(format!("Category {}", id)));
        }

        Ok(())
    }

    async fn search_categories(&self, query: &str) -> Result<Vec<Category>, DomainError> {
        let pattern = format!("%{}%", query);
        let rows: Vec<CategoryRow> = sqlx::query_as(
             r#"SELECT id, name, slug, description, image_url, parent_id, display_order,
                 is_active, target_gender, created_at, updated_at
               FROM categories
               WHERE name ILIKE $1 OR slug ILIKE $1
               ORDER BY display_order ASC, name ASC"#,
        )
        .bind(pattern)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(rows.into_iter().map(CategoryRow::into_category).collect())
    }

}
