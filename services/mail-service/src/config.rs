use anyhow::Result;

#[derive(Debug, Clone)]
pub struct AppConfig {
    // ── SMTP ─────────────────────────────────────────────────────────
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub mail_from_email: String,
    pub mail_from_name: String,

    // ── Transport ────────────────────────────────────────────────────
    /// "smtp" | "console"  — chọn transport adapter
    pub mail_transport: String,

    // ── Kafka ────────────────────────────────────────────────────
    pub kafka_brokers: String,
    pub kafka_topic: String,
    pub kafka_group_id: String,

    // ── Welcome Mail Template ─────────────────────────────────────
    pub welcome_verify_base_url: String,
    pub welcome_cta_text: String,
    pub welcome_product_name: String,
    pub welcome_support_email: String,
    pub welcome_subject: String,

    // ── HTTP ─────────────────────────────────────────────────────────
    pub http_addr: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            smtp_host: std::env::var("SMTP_HOST")
                .unwrap_or_else(|_| "sandbox.smtp.mailtrap.io".into()),
            smtp_port: std::env::var("SMTP_PORT")
                .unwrap_or_else(|_| "2525".into())
                .parse()?,
            smtp_username: std::env::var("SMTP_USERNAME").unwrap_or_else(|_| "".into()),
            smtp_password: std::env::var("SMTP_PASSWORD").unwrap_or_else(|_| "".into()),
            mail_from_email: std::env::var("MAIL_FROM_EMAIL")
                .unwrap_or_else(|_| "noreply@store.local".into()),
            mail_from_name: std::env::var("MAIL_FROM_NAME")
                .unwrap_or_else(|_| "Store Platform".into()),
            mail_transport: std::env::var("MAIL_TRANSPORT").unwrap_or_else(|_| "smtp".into()),
            kafka_brokers: std::env::var("KAFKA_BROKERS")
                .unwrap_or_else(|_| "localhost:9094".into()),
            kafka_topic: std::env::var("KAFKA_TOPIC").unwrap_or_else(|_| "domain-events".into()),
            kafka_group_id: std::env::var("KAFKA_GROUP_ID")
                .unwrap_or_else(|_| "mail-service".into()),
            welcome_verify_base_url: std::env::var("WELCOME_VERIFY_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:3001/api/v1/users/verify".into()),
            welcome_cta_text: std::env::var("WELCOME_CTA_TEXT")
                .unwrap_or_else(|_| "Xac thuc tai khoan".into()),
            welcome_product_name: std::env::var("WELCOME_PRODUCT_NAME")
                .unwrap_or_else(|_| "Store Platform".into()),
            welcome_support_email: std::env::var("WELCOME_SUPPORT_EMAIL")
                .unwrap_or_else(|_| "support@store.local".into()),
            welcome_subject: std::env::var("WELCOME_SUBJECT")
                .unwrap_or_else(|_| "Chao mung ban den voi Store!".into()),
            http_addr: std::env::var("HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:3002".into()),
        })
    }
}
