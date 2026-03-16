use serde::{Deserialize, Serialize};
use validator::Validate;

/// Command: tạo user mới
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateUserCommand {
    #[validate(email(message = "Invalid email"))]
    pub email: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,

    #[validate(length(min = 2, max = 100, message = "Full name must be 2-100 characters"))]
    pub full_name: String,

    pub role: Option<String>, // "admin" | "customer" | "seller"
}
