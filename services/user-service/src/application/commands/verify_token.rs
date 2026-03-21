use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct VerifyTokenCommand {
    /// JWT token cần xác thực
    #[validate(length(min = 1, message = "Token cannot be empty"))]
    pub token: String,
}
