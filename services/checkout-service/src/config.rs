use anyhow::Result;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub http_addr: String,
    pub database_url: String,
    pub service_name: String,
    pub cart_service_base_url: String,
    pub kafka_brokers: String,
    pub kafka_topic: String,
    pub telegram_chat_id: String,
    pub checkout_success_url: String,
    pub checkout_cancel_url: String,
    pub payment_sandbox_mode: bool,
    pub paypal_api_base_url: String,
    pub paypal_api_key: String,
    pub paypal_webhook_secret: String,
    pub stripe_api_base_url: String,
    pub stripe_api_key: String,
    pub stripe_webhook_secret: String,
    pub oxapay_api_base_url: String,
    pub oxapay_api_key: String,
    pub oxapay_webhook_secret: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            http_addr: std::env::var("HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:3004".into()),
            database_url: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://postgres:password@localhost:5432/store_platform".into()
            }),
            service_name: "checkout-service".into(),
            cart_service_base_url: std::env::var("CART_SERVICE_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:3003".into()),
            kafka_brokers: std::env::var("KAFKA_BROKERS")
                .unwrap_or_else(|_| "localhost:9094".into()),
            kafka_topic: std::env::var("KAFKA_TOPIC").unwrap_or_else(|_| "domain-events".into()),
            telegram_chat_id: std::env::var("TELEGRAM_CHAT_ID")
                .or_else(|_| std::env::var("TELEGRAM_CHANNEL_ID"))
                .unwrap_or_default(),
            checkout_success_url: std::env::var("CHECKOUT_SUCCESS_URL")
                .unwrap_or_else(|_| "http://localhost:5173/checkout/success".into()),
            checkout_cancel_url: std::env::var("CHECKOUT_CANCEL_URL")
                .unwrap_or_else(|_| "http://localhost:5173/checkout/cancel".into()),
            payment_sandbox_mode: std::env::var("PAYMENT_SANDBOX_MODE")
                .ok()
                .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
                .unwrap_or(true),
            paypal_api_base_url: std::env::var("PAYPAL_API_BASE_URL")
                .unwrap_or_else(|_| "https://api-m.sandbox.paypal.com".into()),
            paypal_api_key: std::env::var("PAYPAL_API_KEY").unwrap_or_default(),
            paypal_webhook_secret: std::env::var("PAYPAL_WEBHOOK_SECRET").unwrap_or_default(),
            stripe_api_base_url: std::env::var("STRIPE_API_BASE_URL")
                .unwrap_or_else(|_| "https://api.stripe.com".into()),
            stripe_api_key: std::env::var("STRIPE_API_KEY").unwrap_or_default(),
            stripe_webhook_secret: std::env::var("STRIPE_WEBHOOK_SECRET").unwrap_or_default(),
            oxapay_api_base_url: std::env::var("OXAPAY_API_BASE_URL")
                .unwrap_or_else(|_| "https://api.oxapay.com".into()),
            oxapay_api_key: std::env::var("OXAPAY_API_KEY").unwrap_or_default(),
            oxapay_webhook_secret: std::env::var("OXAPAY_WEBHOOK_SECRET").unwrap_or_default(),
        })
    }
}
