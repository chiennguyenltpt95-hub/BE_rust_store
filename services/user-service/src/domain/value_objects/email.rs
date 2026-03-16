use domain_core::error::DomainError;

/// Email Value Object — immutable, tự validate
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Email(String);

impl Email {
    pub fn new(raw: impl Into<String>) -> Result<Self, DomainError> {
        let email = raw.into().trim().to_lowercase();
        if !Self::is_valid(&email) {
            return Err(DomainError::ValidationError(
                format!("Invalid email format: {}", email)
            ));
        }
        Ok(Self(email))
    }

    fn is_valid(email: &str) -> bool {
        // Minimal email validation — production nên dùng `validator` crate
        let parts: Vec<&str> = email.split('@').collect();
        parts.len() == 2
            && !parts[0].is_empty()
            && parts[1].contains('.')
            && !parts[1].starts_with('.')
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
