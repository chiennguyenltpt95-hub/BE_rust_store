use anyhow::Result;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub nats_url: String,
    pub http_addr: String,
    pub jwt_secret: String,
    pub service_name: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://postgres:password@localhost:5432/store_platform".into()
            }),
            nats_url: std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".into()),
            http_addr: std::env::var("HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:3001".into()),
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "super-secret-change-me".into()),
            service_name: "user-service".into(),
        })
    }
}
