use async_trait::async_trait;
use domain_core::error::DomainError;

use crate::application::ports::{PaymentGateway, PaymentRequest, PaymentResult};

pub struct OxaPayGateway {
    client: reqwest::Client,
    api_base_url: String,
    api_key: String,
    sandbox_mode: bool,
}

impl OxaPayGateway {
    pub fn new(api_base_url: String, api_key: String, sandbox_mode: bool) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_base_url,
            api_key,
            sandbox_mode,
        }
    }
}

#[async_trait]
impl PaymentGateway for OxaPayGateway {
    async fn create_payment(&self, request: &PaymentRequest) -> Result<PaymentResult, DomainError> {
        if self.sandbox_mode {
            let id = format!("oxa_sandbox_{}", request.checkout_id);
            let url = format!("https://sandbox.oxapay.test/pay/{}", request.checkout_id);
            return Ok(PaymentResult {
                provider: "oxapay".into(),
                provider_payment_id: id,
                checkout_url: url,
                status: "pending".into(),
                raw_response: serde_json::json!({"mode":"sandbox","provider":"oxapay"}),
            });
        }

        if self.api_key.is_empty() {
            return Err(DomainError::InfrastructureError(
                "OXAPAY_API_KEY is empty".into(),
            ));
        }

        let payload = serde_json::json!({
            "amount": (request.amount_cents as f64) / 100.0,
            "currency": request.currency,
            "description": request.description,
            "callbackUrl": request.success_url,
            "orderId": request.checkout_id.to_string(),
        });

        let resp = self
            .client
            .post(format!(
                "{}/v1/payment/invoice",
                self.api_base_url.trim_end_matches('/')
            ))
            .header("merchant_api_key", &self.api_key)
            .json(&payload)
            .send()
            .await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        let status_code = resp.status();
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| DomainError::InfrastructureError(e.to_string()))?;

        if !status_code.is_success() {
            return Err(DomainError::InfrastructureError(format!(
                "OxaPay create payment failed: {}",
                body
            )));
        }

        let provider_payment_id = body
            .get("trackId")
            .and_then(|v| v.as_str())
            .unwrap_or("oxapay_payment")
            .to_string();
        let checkout_url = body
            .get("payLink")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        Ok(PaymentResult {
            provider: "oxapay".into(),
            provider_payment_id,
            checkout_url,
            status: "pending".into(),
            raw_response: body,
        })
    }
}
