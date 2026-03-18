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
            http_addr: std::env::var("HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:3002".into()),
        })
    }
}
