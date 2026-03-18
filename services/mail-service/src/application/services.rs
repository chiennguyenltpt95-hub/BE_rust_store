use std::sync::Arc;

use domain_core::error::DomainError;
use tracing::{info, instrument};
use validator::Validate;

use crate::application::commands::{SendRawMailCommand, SendTemplatedMailCommand};
use crate::domain::models::{MailAddress, MailMessageBuilder};
use crate::domain::ports::{MailTransport, TemplateEngine};

/// Application Service — điều phối use case gửi mail.
/// Không chứa business logic, chỉ gọi domain objects + ports.
pub struct MailAppService {
    transport: Arc<dyn MailTransport>,
    template_engine: Arc<dyn TemplateEngine>,
    default_from: MailAddress,
}

impl MailAppService {
    pub fn new(
        transport: Arc<dyn MailTransport>,
        template_engine: Arc<dyn TemplateEngine>,
        default_from: MailAddress,
    ) -> Self {
        Self {
            transport,
            template_engine,
            default_from,
        }
    }

    /// Use case: gửi email raw (text/html trực tiếp)
    #[instrument(skip(self, cmd), fields(to = %cmd.to))]
    pub async fn send_raw_mail(&self, cmd: SendRawMailCommand) -> Result<(), DomainError> {
        cmd.validate()
            .map_err(|e| DomainError::ValidationError(e.to_string()))?;

        if cmd.text.is_none() && cmd.html.is_none() {
            return Err(DomainError::ValidationError(
                "Either 'text' or 'html' body is required".into(),
            ));
        }

        let to = match &cmd.to_name {
            Some(name) => MailAddress::with_name(&cmd.to, name),
            None => MailAddress::new(&cmd.to),
        };

        let mut builder = MailMessageBuilder::new()
            .from(self.default_from.clone())
            .to(to)
            .subject(&cmd.subject);

        if let Some(text) = &cmd.text {
            builder = builder.text(text);
        }
        if let Some(html) = &cmd.html {
            builder = builder.html(html);
        }

        let message = builder
            .build()
            .map_err(|e| DomainError::ValidationError(e))?;

        self.transport.send(&message).await?;

        info!("Raw email sent to {}", cmd.to);
        Ok(())
    }

    /// Use case: gửi email với template (Welcome, Reset Password…)
    #[instrument(skip(self, cmd), fields(to = %cmd.to, template = %cmd.template_name))]
    pub async fn send_templated_mail(
        &self,
        cmd: SendTemplatedMailCommand,
    ) -> Result<(), DomainError> {
        cmd.validate()
            .map_err(|e| DomainError::ValidationError(e.to_string()))?;

        // Render template → HTML
        let html = self
            .template_engine
            .render(&cmd.template_name, &cmd.context)?;

        let to = match &cmd.to_name {
            Some(name) => MailAddress::with_name(&cmd.to, name),
            None => MailAddress::new(&cmd.to),
        };

        let message = MailMessageBuilder::new()
            .from(self.default_from.clone())
            .to(to)
            .subject(&cmd.subject)
            .html(html)
            .build()
            .map_err(|e| DomainError::ValidationError(e))?;

        self.transport.send(&message).await?;

        info!("Templated email '{}' sent to {}", cmd.template_name, cmd.to);
        Ok(())
    }
}
