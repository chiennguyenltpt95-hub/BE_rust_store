use anyhow::Result;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub http_addr: String,
    pub database_url: String,
    pub kafka_brokers: String,
    pub kafka_topic: String,
    pub kafka_group_id: String,
    pub telegram_bot_token: String,
    pub telegram_channel_id: String,
    pub worker_poll_interval_secs: u64,
    pub worker_batch_size: i64,
    pub default_max_attempts: i32,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            http_addr: std::env::var("HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:3008".into()),
            database_url: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://postgres:password@localhost:5432/store_platform".into()
            }),
            kafka_brokers: std::env::var("KAFKA_BROKERS")
                .unwrap_or_else(|_| "localhost:9094".into()),
            kafka_topic: std::env::var("KAFKA_TOPIC").unwrap_or_else(|_| "domain-events".into()),
            kafka_group_id: std::env::var("KAFKA_GROUP_ID")
                .unwrap_or_else(|_| "notification-service".into()),
            telegram_bot_token: std::env::var("TELEGRAM_BOT_TOKEN").unwrap_or_default(),
            telegram_channel_id: std::env::var("TELEGRAM_CHANNEL_ID").unwrap_or_default(),
            worker_poll_interval_secs: std::env::var("WORKER_POLL_INTERVAL_SECS")
                .ok()
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(3),
            worker_batch_size: std::env::var("WORKER_BATCH_SIZE")
                .ok()
                .and_then(|v| v.parse::<i64>().ok())
                .unwrap_or(50),
            default_max_attempts: std::env::var("DEFAULT_MAX_ATTEMPTS")
                .ok()
                .and_then(|v| v.parse::<i32>().ok())
                .unwrap_or(5),
        })
    }
}
