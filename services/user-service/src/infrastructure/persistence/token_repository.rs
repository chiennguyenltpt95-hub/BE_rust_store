use async_trait::async_trait;
use chrono::Utc;
use domain_core::error::DomainError;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::repositories::token_repository::{RefreshToken, TokenRepository};

pub struct PgTokenRepository {
    pool: PgPool,
}

impl PgTokenRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct TokenRow {
    id: Uuid,
    user_id: Uuid,
    token_hash: String,
    expires_at: chrono::DateTime<chrono::Utc>,
    created_at: chrono::DateTime<chrono::Utc>,
    revoked_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<TokenRow> for RefreshToken {
    fn from(r: TokenRow) -> Self {
        RefreshToken {
            id: r.id,
            user_id: r.user_id,
            token_hash: r.token_hash,
            expires_at: r.expires_at,
            created_at: r.created_at,
            revoked_at: r.revoked_at,
        }
    }
}

#[async_trait]
impl TokenRepository for PgTokenRepository {
    async fn save(&self, token: &RefreshToken) -> Result<(), DomainError> {
        sqlx::query(
            r#"INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at, created_at)
               VALUES ($1, $2, $3, $4, $5)"#,
        )
        .bind(token.id)
        .bind(token.user_id)
        .bind(&token.token_hash)
        .bind(token.expires_at)
        .bind(token.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;
        Ok(())
    }

    async fn find_by_hash(&self, hash: &str) -> Result<Option<RefreshToken>, DomainError> {
        let row: Option<TokenRow> = sqlx::query_as(
            r#"SELECT id, user_id, token_hash, expires_at, created_at, revoked_at
               FROM refresh_tokens WHERE token_hash = $1"#,
        )
        .bind(hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        Ok(row.map(RefreshToken::from))
    }

    async fn revoke_by_hash(&self, hash: &str) -> Result<(), DomainError> {
        sqlx::query("UPDATE refresh_tokens SET revoked_at = $1 WHERE token_hash = $2")
            .bind(Utc::now())
            .bind(hash)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;
        Ok(())
    }

    async fn revoke_all_for_user(&self, user_id: Uuid) -> Result<(), DomainError> {
        sqlx::query(
            "UPDATE refresh_tokens SET revoked_at = $1 WHERE user_id = $2 AND revoked_at IS NULL",
        )
        .bind(Utc::now())
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;
        Ok(())
    }
}
