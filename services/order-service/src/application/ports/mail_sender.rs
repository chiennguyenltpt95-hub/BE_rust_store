#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MailMessage {
    pub to: String,
    pub to_name: Option<String>,
    pub subject: String,
    pub template_name: String,
    pub context: serde_json::Value,
}
