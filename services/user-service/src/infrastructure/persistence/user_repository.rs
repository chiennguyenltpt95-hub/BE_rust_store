use async_trait::async_trait;
use domain_core::error::DomainError;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::{
    user::{UserRole, UserStatus},
    User,
};
use crate::domain::repositories::UserRepository;
use crate::domain::value_objects::{Email, HashedPassword};

/// Postgres implementation của UserRepository port
pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Row mapping từ DB
#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    email: String,
    password_hash: String,
    full_name: String,
    role: String,
    status: String,
    address: Option<String>,
    age: Option<i16>,
    wallet_address: Option<String>,
    verified: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<UserRow> for User {
    type Error = DomainError;

    fn try_from(row: UserRow) -> Result<Self, Self::Error> {
        let email = Email::new(&row.email)?;
        let password = HashedPassword::from_hash(row.password_hash);
        let role = match row.role.as_str() {
            "admin" => UserRole::Admin,
            "seller" => UserRole::Seller,
            _ => UserRole::Customer,
        };
        let status = match row.status.as_str() {
            "inactive" => UserStatus::Inactive,
            "banned" => UserStatus::Banned,
            _ => UserStatus::Active,
        };
        Ok(User::reconstitute(
            row.id,
            email,
            password,
            row.full_name,
            role,
            status,
            row.address,
            row.age,
            row.wallet_address,
            row.verified,
            row.created_at,
            row.updated_at,
        ))
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError> {
        let row: Option<UserRow> = sqlx::query_as(
            r#"SELECT id, email, password_hash, full_name,
                      role::text, status::text,
                      address, age, wallet_address, verified,
                      created_at, updated_at
               FROM users WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        row.map(User::try_from).transpose()
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, DomainError> {
        let row: Option<UserRow> = sqlx::query_as(
            r#"SELECT id, email, password_hash, full_name,
                      role::text, status::text,
                      address, age, wallet_address, verified,
                      created_at, updated_at
               FROM users WHERE email = $1"#,
        )
        .bind(email.value())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        row.map(User::try_from).transpose()
    }

    async fn save(&self, user: &User) -> Result<(), DomainError> {
        let role_str = format!("{:?}", user.role).to_lowercase();
        let status_str = format!("{:?}", user.status).to_lowercase();

        sqlx::query(
            r#"INSERT INTO users
               (id, email, password_hash, full_name, role, status, address, age, wallet_address, verified, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5::user_role, $6::user_status, $7, $8, $9, $10, $11, $12)"#,
        )
        .bind(user.id)
        .bind(user.email.value())
        .bind(user.password.hash())
        .bind(&user.full_name)
        .bind(&role_str)
        .bind(&status_str)
        .bind(&user.address)
        .bind(user.age)
        .bind(&user.wallet_address)
        .bind(user.verified)
        .bind(user.created_at)
        .bind(user.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(())
    }

    async fn update(&self, user: &User) -> Result<(), DomainError> {
        let status_str = format!("{:?}", user.status).to_lowercase();

        sqlx::query(
            r#"UPDATE users
               SET full_name = $2, status = $3::user_status,
                   address = $4, age = $5, wallet_address = $6,
                   verified = $7, updated_at = $8
               WHERE id = $1"#,
        )
        .bind(user.id)
        .bind(&user.full_name)
        .bind(&status_str)
        .bind(&user.address)
        .bind(user.age)
        .bind(&user.wallet_address)
        .bind(user.verified)
        .bind(user.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(())
    }

    async fn set_verified(&self, id: Uuid) -> Result<(), DomainError> {
        sqlx::query(
            r#"UPDATE users SET verified = true, updated_at = NOW() WHERE id = $1"#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;
        Ok(())
    }

    async fn exists_by_email(&self, email: &Email) -> Result<bool, DomainError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE email = $1")
            .bind(email.value())
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;
        Ok(count.0 > 0)
    }
}
