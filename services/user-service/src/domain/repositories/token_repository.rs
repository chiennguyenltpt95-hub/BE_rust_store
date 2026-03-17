use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain_core::error::DomainError;
use uuid::Uuid;

/// Token record lưu trong DB
#[derive(Debug)]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

impl RefreshToken {
    pub fn is_valid(&self) -> bool {
        self.revoked_at.is_none() && self.expires_at > Utc::now()
    }
}

/// Port — contract cho token repository
#[async_trait]
pub trait TokenRepository: Send + Sync {
    async fn save(&self, token: &RefreshToken) -> Result<(), DomainError>;
    async fn find_by_hash(&self, hash: &str) -> Result<Option<RefreshToken>, DomainError>;
    async fn revoke_by_hash(&self, hash: &str) -> Result<(), DomainError>;
    async fn revoke_all_for_user(&self, user_id: Uuid) -> Result<(), DomainError>;
}
