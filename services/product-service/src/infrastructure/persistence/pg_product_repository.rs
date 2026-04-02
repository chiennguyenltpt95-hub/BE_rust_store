use async_trait::async_trait;
use domain_core::error::DomainError;
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
    category_id: Option<Uuid>,
    category_name: Option<String>,
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
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl CategoryRow {
    fn into_category(self) -> Category {
        Category {
            id: self.id,
            name: self.name,
            slug: self.slug,
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
            category_id: self.category_id,
            category_name: self.category_name,
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
            r#"SELECT p.id, p.name, p.description, p.category_id, c.name AS category_name,
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
             (id, name, description, category_id, price_cents, stock, status, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
        )
        .bind(product.id)
        .bind(&product.name)
        .bind(&product.description)
         .bind(product.category_id)
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
                   category_id = $4,
                   price_cents = $5,
                   stock = $6,
                   status = $7,
                   updated_at = $8
               WHERE id = $1"#,
        )
        .bind(product.id)
        .bind(&product.name)
        .bind(&product.description)
        .bind(product.category_id)
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
            r#"SELECT p.id, p.name, p.description, p.category_id, c.name AS category_name,
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
            r#"INSERT INTO categories (id, name, slug, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5)"#,
        )
        .bind(category.id)
        .bind(&category.name)
        .bind(&category.slug)
        .bind(category.created_at)
        .bind(category.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(())
    }

    async fn find_category_by_id(&self, id: Uuid) -> Result<Option<Category>, DomainError> {
        let row: Option<CategoryRow> = sqlx::query_as(
            r#"SELECT id, name, slug, created_at, updated_at
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
            r#"SELECT id, name, slug, created_at, updated_at
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
            r#"SELECT id, name, slug, created_at, updated_at
               FROM categories
               ORDER BY name ASC"#,
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
}
