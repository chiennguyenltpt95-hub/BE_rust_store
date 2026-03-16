use domain_core::error::DomainError;

/// HashedPassword Value Object
/// Chỉ lưu hash, không bao giờ expose raw password.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HashedPassword(String);

impl HashedPassword {
    /// Hash password mới (dùng khi tạo user)
    pub fn from_raw(raw: &str) -> Result<Self, DomainError> {
        if raw.len() < 8 {
            return Err(DomainError::ValidationError(
                "Password must be at least 8 characters".into(),
            ));
        }
        let hash = bcrypt::hash(raw, bcrypt::DEFAULT_COST)
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;
        Ok(Self(hash))
    }

    /// Wrap hash đã có sẵn từ DB
    pub fn from_hash(hash: String) -> Self {
        Self(hash)
    }

    /// Verify raw password với hash
    pub fn verify(&self, raw: &str) -> bool {
        bcrypt::verify(raw, &self.0).unwrap_or(false)
    }

    pub fn hash(&self) -> &str {
        &self.0
    }
}
