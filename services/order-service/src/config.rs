use anyhow::Result;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub http_addr: String,
    pub database_url: String,
    pub service_name: String,
    pub checkout_service_base_url: String,
    pub kafka_brokers: String,
    pub kafka_topic: String,
    pub outbox_poll_interval_secs: u64,
    pub outbox_batch_size: i64,
    pub outbox_max_attempts: i32,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            http_addr: std::env::var("HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:3005".into()),
            database_url: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://postgres:password@localhost:5432/store_platform".into()
            }),
            service_name: "order-service".into(),
            checkout_service_base_url: std::env::var("CHECKOUT_SERVICE_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:3004".into()),
            kafka_brokers: std::env::var("KAFKA_BROKERS")
                .unwrap_or_else(|_| "localhost:9094".into()),
            kafka_topic: std::env::var("KAFKA_TOPIC").unwrap_or_else(|_| "domain-events".into()),
            outbox_poll_interval_secs: std::env::var("OUTBOX_POLL_INTERVAL_SECS")
                .ok()
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(5),
            outbox_batch_size: std::env::var("OUTBOX_BATCH_SIZE")
                .ok()
                .and_then(|v| v.parse::<i64>().ok())
                .unwrap_or(20),
            outbox_max_attempts: std::env::var("OUTBOX_MAX_ATTEMPTS")
                .ok()
                .and_then(|v| v.parse::<i32>().ok())
                .unwrap_or(5),
        })
    }
}
