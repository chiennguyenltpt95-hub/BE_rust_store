use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Entity not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Business rule violation: {0}")]
    BusinessRuleViolation(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Infrastructure error: {0}")]
    InfrastructureError(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),
}
