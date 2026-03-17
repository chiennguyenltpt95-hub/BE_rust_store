use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// Login — trả access token + refresh token
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginCommand {
    #[validate(email(message = "Invalid email"))]
    #[schema(example = "user@example.com")]
    pub email: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    #[schema(example = "password123")]
    pub password: String,
}

/// Dùng refresh token để lấy access token mới
#[derive(Debug, Deserialize, ToSchema)]
pub struct RefreshTokenCommand {
    pub refresh_token: String,
}

/// Logout — thu hồi refresh token
#[derive(Debug, Deserialize, ToSchema)]
pub struct LogoutCommand {
    pub refresh_token: String,
}

/// Response trả về sau khi login / refresh
#[derive(Debug, Serialize, ToSchema)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64, // seconds
}
