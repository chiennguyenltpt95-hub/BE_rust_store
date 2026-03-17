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
}
