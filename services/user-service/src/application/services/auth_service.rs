use chrono::{Duration, Utc};
use domain_core::error::DomainError;
use std::sync::Arc;
use tracing::instrument;
use uuid::Uuid;
use validator::Validate;

use crate::application::commands::auth::{
    LoginCommand, LogoutCommand, RefreshTokenCommand, TokenPair,
};
use crate::domain::repositories::token_repository::{RefreshToken, TokenRepository};
use crate::domain::repositories::UserRepository;
use crate::domain::value_objects::Email;
use crate::infrastructure::auth::{
    generate_refresh_token, hash_token, JwtService, ACCESS_TOKEN_TTL_SECS, REFRESH_TOKEN_TTL_DAYS,
};

pub struct AuthAppService {
    user_repo: Arc<dyn UserRepository>,
    token_repo: Arc<dyn TokenRepository>,
    jwt: JwtService,
}

impl AuthAppService {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        token_repo: Arc<dyn TokenRepository>,
        jwt_secret: &str,
    ) -> Self {
        Self {
            user_repo,
            token_repo,
            jwt: JwtService::new(jwt_secret),
        }
    }

    // ── LOGIN ────────────────────────────────────────────────────────────────

    #[instrument(skip(self, cmd))]
    pub async fn login(&self, cmd: LoginCommand) -> Result<TokenPair, DomainError> {
        cmd.validate()
            .map_err(|e| DomainError::ValidationError(e.to_string()))?;

        let email = Email::new(&cmd.email)?;
        let user = self
            .user_repo
            .find_by_email(&email)
            .await?
            .ok_or_else(|| DomainError::Unauthorized("Invalid email or password".into()))?;

        // Verify password
        if !user.password.verify(&cmd.password) {
            return Err(DomainError::Unauthorized(
                "Invalid email or password".into(),
            ));
        }

        self.issue_token_pair(user.id, &format!("{:?}", user.role))
            .await
    }

    // ── REFRESH ──────────────────────────────────────────────────────────────

    #[instrument(skip(self, cmd))]
    pub async fn refresh(&self, cmd: RefreshTokenCommand) -> Result<TokenPair, DomainError> {
        let hash = hash_token(&cmd.refresh_token);

        let stored = self
            .token_repo
            .find_by_hash(&hash)
            .await?
            .ok_or_else(|| DomainError::Unauthorized("Refresh token not found".into()))?;

        if !stored.is_valid() {
            return Err(DomainError::Unauthorized(
                "Refresh token expired or revoked".into(),
            ));
        }

        // Rotate: thu hồi token cũ, cấp token mới
        self.token_repo.revoke_by_hash(&hash).await?;

        let user = self
            .user_repo
            .find_by_id(stored.user_id)
            .await?
            .ok_or_else(|| DomainError::NotFound("User not found".into()))?;

        self.issue_token_pair(user.id, &format!("{:?}", user.role))
            .await
    }

    // ── LOGOUT ───────────────────────────────────────────────────────────────

    #[instrument(skip(self, cmd))]
    pub async fn logout(&self, cmd: LogoutCommand) -> Result<(), DomainError> {
        let hash = hash_token(&cmd.refresh_token);
        self.token_repo.revoke_by_hash(&hash).await
    }

    // ── HELPERS ──────────────────────────────────────────────────────────────

    async fn issue_token_pair(&self, user_id: Uuid, role: &str) -> Result<TokenPair, DomainError> {
        let access_token = self.jwt.generate_access_token(user_id, role)?;

        let raw_refresh = generate_refresh_token();
        let token_hash = hash_token(&raw_refresh);
        let now = Utc::now();

        let record = RefreshToken {
            id: Uuid::new_v4(),
            user_id,
            token_hash,
            expires_at: now + Duration::days(REFRESH_TOKEN_TTL_DAYS),
            created_at: now,
            revoked_at: None,
        };
        self.token_repo.save(&record).await?;

        Ok(TokenPair {
            access_token,
            refresh_token: raw_refresh,
            expires_in: ACCESS_TOKEN_TTL_SECS,
        })
    }
}
