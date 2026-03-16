use chrono::{DateTime, Utc};
use domain_core::aggregate::AggregateBase;
use domain_core::error::DomainError;
use uuid::Uuid;

use crate::domain::value_objects::{Email, HashedPassword};
use crate::domain::events::user_events::{UserCreated, UserUpdated, UserDeleted};

/// User Role
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Customer,
    Seller,
}

/// User Status
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_status", rename_all = "lowercase")]
pub enum UserStatus {
    Active,
    Inactive,
    Banned,
}

/// User — Aggregate Root
#[derive(Debug)]
pub struct User {
    pub id: Uuid,
    pub email: Email,
    pub password: HashedPassword,
    pub full_name: String,
    pub role: UserRole,
    pub status: UserStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    base: AggregateBase,
}

impl User {
    /// Factory method — tạo user mới (dùng qua Command handler)
    pub fn create(
        email: Email,
        password: HashedPassword,
        full_name: String,
        role: UserRole,
    ) -> Result<Self, DomainError> {
        if full_name.trim().is_empty() {
            return Err(DomainError::ValidationError("Full name cannot be empty".into()));
        }

        let id = Uuid::new_v4();
        let now = Utc::now();

        let mut base = AggregateBase::new();
        base.record_event(Box::new(UserCreated {
            user_id: id,
            email: email.value().to_string(),
            full_name: full_name.clone(),
            role: format!("{:?}", role),
            occurred_at: now,
        }));

        Ok(Self {
            id,
            email,
            password,
            full_name,
            role,
            status: UserStatus::Active,
            created_at: now,
            updated_at: now,
            base,
        })
    }

    /// Reconstruct từ persistence (không raise event)
    pub fn reconstitute(
        id: Uuid,
        email: Email,
        password: HashedPassword,
        full_name: String,
        role: UserRole,
        status: UserStatus,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            email,
            password,
            full_name,
            role,
            status,
            created_at,
            updated_at,
            base: AggregateBase::new(),
        }
    }

    /// Business method: cập nhật thông tin
    pub fn update_profile(&mut self, full_name: String) -> Result<(), DomainError> {
        if full_name.trim().is_empty() {
            return Err(DomainError::ValidationError("Full name cannot be empty".into()));
        }
        self.full_name = full_name;
        self.updated_at = Utc::now();
        self.base.record_event(Box::new(UserUpdated {
            user_id: self.id,
            full_name: self.full_name.clone(),
            occurred_at: self.updated_at,
        }));
        Ok(())
    }

    /// Business method: xóa mềm / ban user
    pub fn deactivate(&mut self) -> Result<(), DomainError> {
        if self.status == UserStatus::Banned {
            return Err(DomainError::BusinessRuleViolation("User already banned".into()));
        }
        self.status = UserStatus::Inactive;
        self.updated_at = Utc::now();
        self.base.record_event(Box::new(UserDeleted {
            user_id: self.id,
            occurred_at: self.updated_at,
        }));
        Ok(())
    }

    pub fn uncommitted_events(&self) -> &Vec<Box<dyn domain_core::domain_event::DomainEvent>> {
        self.base.uncommitted_events()
    }

    pub fn mark_events_committed(&mut self) {
        self.base.mark_committed();
    }
}
