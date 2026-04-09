use anyhow::Result;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub http_addr: String,
    pub service_name: String,
    pub jwt_secret: String,
    pub aws_region: String,
    pub s3_bucket: String,
    pub s3_key_prefix: String,
    pub s3_presign_expires_seconds: u64,
    pub s3_endpoint_url: Option<String>,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        let s3_bucket = std::env::var("S3_BUCKET")
            .or_else(|_| std::env::var("S3_BUCKET_NAME"))
            .unwrap_or_else(|_| "store-assets".into());

        Ok(Self {
            http_addr: std::env::var("HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:3011".into()),
            service_name: "upload-service".into(),
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "super-secret-change-me".into()),
            aws_region: std::env::var("AWS_REGION").unwrap_or_else(|_| "ap-southeast-1".into()),
            s3_bucket,
            s3_key_prefix: std::env::var("S3_KEY_PREFIX").unwrap_or_else(|_| "products".into()),
            s3_presign_expires_seconds: std::env::var("S3_PRESIGN_EXPIRES_SECONDS")
                .ok()
                .and_then(|v| v.parse::<u64>().ok())
                .filter(|v| *v > 0)
                .unwrap_or(900),
            s3_endpoint_url: std::env::var("S3_ENDPOINT_URL").ok(),
        })
    }
}
