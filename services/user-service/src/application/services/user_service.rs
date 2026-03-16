use std::sync::Arc;
use domain_core::error::DomainError;
use tracing::{info, instrument};
use uuid::Uuid;
use validator::Validate;

use crate::domain::entities::{User, user::UserRole};
use crate::domain::repositories::UserRepository;
use crate::domain::value_objects::{Email, HashedPassword};
use crate::application::commands::{CreateUserCommand, UpdateUserCommand, DeleteUserCommand};
use crate::application::queries::get_user::UserView;
use crate::application::queries::list_users::{ListUsersQuery, UserSummary};
use crate::infrastructure::messaging::EventPublisher;

/// Application Service (Use Case orchestrator)
/// Không chứa business logic — chỉ điều phối domain objects và ports.
pub struct UserAppService {
    user_repo: Arc<dyn UserRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl UserAppService {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self { user_repo, event_publisher }
    }

    // ── COMMANDS ────────────────────────────────────────────────────────────

    #[instrument(skip(self, cmd))]
    pub async fn create_user(&self, cmd: CreateUserCommand) -> Result<Uuid, DomainError> {
        cmd.validate()
            .map_err(|e| DomainError::ValidationError(e.to_string()))?;

        let email = Email::new(&cmd.email)?;

        // Business rule: email phải unique
        if self.user_repo.exists_by_email(&email).await? {
            return Err(DomainError::Conflict(format!(
                "Email {} already registered", cmd.email
            )));
        }

        let password = HashedPassword::from_raw(&cmd.password)?;
        let role = match cmd.role.as_deref() {
            Some("admin") => UserRole::Admin,
            Some("seller") => UserRole::Seller,
            _ => UserRole::Customer,
        };

        let mut user = User::create(email, password, cmd.full_name, role)?;

        // Persist
        self.user_repo.save(&user).await?;

        // Publish domain events
        for event in user.uncommitted_events() {
            self.event_publisher.publish(event.as_ref()).await?;
        }
        user.mark_events_committed();

        info!(user_id = %user.id, "User created");
        Ok(user.id)
    }

    #[instrument(skip(self))]
    pub async fn update_user(&self, cmd: UpdateUserCommand) -> Result<(), DomainError> {
        cmd.validate()
            .map_err(|e| DomainError::ValidationError(e.to_string()))?;

        let mut user = self.user_repo.find_by_id(cmd.user_id).await?
            .ok_or_else(|| DomainError::NotFound(format!("User {}", cmd.user_id)))?;

        user.update_profile(cmd.full_name)?;
        self.user_repo.update(&user).await?;

        for event in user.uncommitted_events() {
            self.event_publisher.publish(event.as_ref()).await?;
        }
        user.mark_events_committed();

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn delete_user(&self, cmd: DeleteUserCommand) -> Result<(), DomainError> {
        let mut user = self.user_repo.find_by_id(cmd.user_id).await?
            .ok_or_else(|| DomainError::NotFound(format!("User {}", cmd.user_id)))?;

        user.deactivate()?;
        self.user_repo.update(&user).await?;

        for event in user.uncommitted_events() {
            self.event_publisher.publish(event.as_ref()).await?;
        }
        user.mark_events_committed();

        Ok(())
    }

    // ── QUERIES ─────────────────────────────────────────────────────────────

    #[instrument(skip(self))]
    pub async fn get_user(&self, user_id: Uuid) -> Result<UserView, DomainError> {
        let user = self.user_repo.find_by_id(user_id).await?
            .ok_or_else(|| DomainError::NotFound(format!("User {}", user_id)))?;

        Ok(UserView {
            id: user.id,
            email: user.email.value().to_string(),
            full_name: user.full_name,
            role: format!("{:?}", user.role),
            status: format!("{:?}", user.status),
            created_at: user.created_at,
        })
    }

    #[instrument(skip(self))]
    pub async fn list_users(&self, _query: ListUsersQuery) -> Result<Vec<UserSummary>, DomainError> {
        // Placeholder — production dùng ReadRepository với filter / pagination
        Ok(vec![])
    }
}
