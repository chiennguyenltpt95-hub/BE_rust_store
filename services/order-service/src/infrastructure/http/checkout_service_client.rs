use async_trait::async_trait;
use domain_core::error::DomainError;
use serde::Deserialize;
use uuid::Uuid;

use crate::application::ports::checkout_reader::CheckoutSnapshot;
use crate::application::ports::CheckoutReaderPort;

pub struct CheckoutServiceClient {
    http: reqwest::Client,
    base_url: String,
}

impl CheckoutServiceClient {
    pub fn new(base_url: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct CheckoutView {
    amount_cents: i64,
    currency: String,
    status: String,
}

#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

#[async_trait]
impl CheckoutReaderPort for CheckoutServiceClient {
    async fn get_checkout(&self, checkout_id: Uuid) -> Result<CheckoutSnapshot, DomainError> {
        let url = format!("{}/api/v1/checkouts/{}", self.base_url, checkout_id);
        let resp = self.http.get(url).send().await.map_err(|e| {
            DomainError::InfrastructureError(format!("Checkout service request failed: {}", e))
        })?;

        if !resp.status().is_success() {
            return Err(DomainError::NotFound(format!("Checkout {}", checkout_id)));
        }

        let body: ApiResponse<CheckoutView> = resp.json().await.map_err(|e| {
            DomainError::InfrastructureError(format!("Invalid checkout response: {}", e))
        })?;

        if !body.success {
            return Err(DomainError::BusinessRuleViolation(
                body.error
                    .unwrap_or_else(|| "Unable to read checkout".to_string()),
            ));
        }

        let data = body
            .data
            .ok_or_else(|| DomainError::InfrastructureError("Missing checkout data".into()))?;

        Ok(CheckoutSnapshot {
            amount_cents: data.amount_cents,
            currency: data.currency,
            status: data.status,
        })
    }
}
