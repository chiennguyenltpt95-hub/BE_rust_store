use async_trait::async_trait;
use domain_core::error::DomainError;
use serde::Deserialize;
use uuid::Uuid;

use crate::application::ports::ProductPricingPort;

pub struct ProductServiceClient {
    http: reqwest::Client,
    base_url: String,
}

impl ProductServiceClient {
    pub fn new(base_url: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ProductView {
    price_cents: i64,
}

#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

#[async_trait]
impl ProductPricingPort for ProductServiceClient {
    async fn get_product_price_cents(&self, product_id: Uuid) -> Result<i64, DomainError> {
        let url = format!("{}/api/v1/products/{}", self.base_url, product_id);

        let resp = self
            .http
            .get(url)
            .send()
            .await
            .map_err(|e| DomainError::InfrastructureError(format!("Product service request failed: {}", e)))?;

        if !resp.status().is_success() {
            return Err(DomainError::NotFound(format!("Product {}", product_id)));
        }

        let body: ApiResponse<ProductView> = resp
            .json()
            .await
            .map_err(|e| DomainError::InfrastructureError(format!("Invalid product service response: {}", e)))?;

        if !body.success {
            return Err(DomainError::BusinessRuleViolation(
                body.error.unwrap_or_else(|| "Product is not available".to_string()),
            ));
        }

        let data = body
            .data
            .ok_or_else(|| DomainError::InfrastructureError("Missing product data".into()))?;

        Ok(data.price_cents)
    }
}
