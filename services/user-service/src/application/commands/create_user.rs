use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

fn validate_password_uppercase(password: &str) -> Result<(), validator::ValidationError> {
    if password.chars().any(|c| c.is_uppercase()) {
        Ok(())
    } else {
        Err(validator::ValidationError::new(
            "Password must contain at least one uppercase letter",
        ))
    }
}

/// Command: tạo user mới
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateUserCommand {
    /// Email hợp lệ
    #[validate(email(message = "Invalid email"))]
    pub email: String,

    /// Tối thiểu 8 ký tự và ít nhất 1 chữ hoa
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    #[validate(custom(function = "validate_password_uppercase"))]
    pub password: String,

    /// 2-100 ký tự
    #[validate(length(min = 2, max = 100, message = "Full name must be 2-100 characters"))]
    pub full_name: String,
}
