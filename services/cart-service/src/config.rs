use anyhow::Result;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub http_addr: String,
    pub database_url: String,
    pub product_service_base_url: String,
    pub service_name: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            http_addr: std::env::var("HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:3003".into()),
            database_url: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://postgres:password@localhost:5432/store_platform".into()
            }),
            product_service_base_url: std::env::var("PRODUCT_SERVICE_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:3002".into()),
            service_name: "cart-service".into(),
        })
    }
}
