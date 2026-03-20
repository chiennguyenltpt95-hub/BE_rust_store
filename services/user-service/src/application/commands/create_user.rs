use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;



/// Command: tạo user mới
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateUserCommand {
    /// Email hợp lệ
    #[validate(email(message = "Invalid email"))]
    pub email: String,

    /// Tối thiểu 8 ký tự
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,

    /// 2-100 ký tự
    #[validate(length(min = 2, max = 100, message = "Full name must be 2-100 characters"))]
    pub full_name: String,

}
