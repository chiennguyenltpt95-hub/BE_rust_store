use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// Command: gửi email trực tiếp (không qua template)
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct SendRawMailCommand {
    #[validate(email(message = "Invalid 'to' email"))]
    pub to: String,

    pub to_name: Option<String>,

    #[validate(length(min = 1, message = "Subject cannot be empty"))]
    pub subject: String,

    /// Nội dung text thuần
    pub text: Option<String>,

    /// Nội dung HTML
    pub html: Option<String>,
}

/// Command: gửi email dùng template (Welcome, Reset Password…)
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct SendTemplatedMailCommand {
    #[validate(email(message = "Invalid 'to' email"))]
    pub to: String,

    pub to_name: Option<String>,

    #[validate(length(min = 1, message = "Template name cannot be empty"))]
    pub template_name: String,

    #[validate(length(min = 1, message = "Subject cannot be empty"))]
    pub subject: String,

    /// Dữ liệu truyền vào template (key-value)
    pub context: serde_json::Value,
}
