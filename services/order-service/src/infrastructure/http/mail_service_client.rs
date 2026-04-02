use async_trait::async_trait;
use domain_core::error::DomainError;

use crate::application::ports::{MailMessage, MailSenderPort};

pub struct MailServiceClient {
    http: reqwest::Client,
    base_url: String,
}

impl MailServiceClient {
    pub fn new(base_url: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }
}

#[derive(serde::Serialize)]
struct SendTemplatedMailRequest {
    to: String,
    to_name: Option<String>,
    template_name: String,
    subject: String,
    context: serde_json::Value,
}

#[async_trait]
impl MailSenderPort for MailServiceClient {
    async fn send_template(&self, mail: MailMessage) -> Result<(), DomainError> {
        let req = SendTemplatedMailRequest {
            to: mail.to,
            to_name: mail.to_name,
            template_name: mail.template_name,
            subject: mail.subject,
            context: mail.context,
        };

        let resp = self
            .http
            .post(format!("{}/api/v1/mail/send-template", self.base_url))
            .json(&req)
            .send()
            .await
            .map_err(|e| {
                DomainError::InfrastructureError(format!("Mail service request failed: {}", e))
            })?;

        if !resp.status().is_success() {
            return Err(DomainError::InfrastructureError(format!(
                "Mail service returned status {}",
                resp.status()
            )));
        }

        Ok(())
    }
}
