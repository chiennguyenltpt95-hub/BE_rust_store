use async_trait::async_trait;
use domain_core::error::DomainError;
use serde::Deserialize;
use uuid::Uuid;

use crate::application::ports::CartReaderPort;

pub struct CartServiceClient {
    http: reqwest::Client,
    base_url: String,
}

impl CartServiceClient {
    pub fn new(base_url: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct CartView {
    total_cents: i64,
}

#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

#[async_trait]
impl CartReaderPort for CartServiceClient {
    async fn get_cart_total_cents(&self, cart_id: Uuid) -> Result<i64, DomainError> {
        let url = format!("{}/api/v1/carts/{}", self.base_url, cart_id);

        let resp = self.http.get(url).send().await.map_err(|e| {
            DomainError::InfrastructureError(format!("Cart service request failed: {}", e))
        })?;

        if !resp.status().is_success() {
            return Err(DomainError::NotFound(format!("Cart {}", cart_id)));
        }

        let body: ApiResponse<CartView> = resp.json().await.map_err(|e| {
            DomainError::InfrastructureError(format!("Invalid cart service response: {}", e))
        })?;

        if !body.success {
            return Err(DomainError::BusinessRuleViolation(
                body.error
                    .unwrap_or_else(|| "Unable to read cart".to_string()),
            ));
        }

        let data = body
            .data
            .ok_or_else(|| DomainError::InfrastructureError("Missing cart data".into()))?;

        if data.total_cents <= 0 {
            return Err(DomainError::BusinessRuleViolation(
                "Cart total must be greater than 0 for checkout".into(),
            ));
        }

        Ok(data.total_cents)
    }
}
