use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateUserCommand {
    #[serde(default)]
    #[schema(value_type = String, example = "00000000-0000-0000-0000-000000000000")]
    pub user_id: Uuid,

    /// 2-100 ký tự
    #[validate(length(min = 2, max = 100, message = "Full name must be 2-100 characters"))]
    pub full_name: String,

    /// Địa chỉ (tối đa 500 ký tự)
    #[validate(length(max = 500, message = "Address must be at most 500 characters"))]
    pub address: Option<String>,

    /// Tuổi (0-150)
    #[validate(range(min = 0, max = 150, message = "Age must be between 0 and 150"))]
    pub age: Option<i16>,

    /// Địa chỉ ví (tối đa 255 ký tự)
    #[validate(length(max = 255, message = "Wallet address must be at most 255 characters"))]
    pub wallet_address: Option<String>,
}
