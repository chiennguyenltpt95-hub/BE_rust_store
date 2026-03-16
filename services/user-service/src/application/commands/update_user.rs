use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateUserCommand {
    pub user_id: Uuid,

    #[validate(length(min = 2, max = 100, message = "Full name must be 2-100 characters"))]
    pub full_name: String,
}
