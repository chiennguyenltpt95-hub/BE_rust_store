use async_trait::async_trait;
use domain_core::error::DomainError;
use lettre::message::{header::ContentType, Mailbox, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use tracing::info;

use crate::domain::models::{MailAddress, MailMessage};
use crate::domain::ports::MailTransport;

/// ═══════════════════════════════════════════════════════════════════════
/// ADAPTER: SmtpTransport — gửi mail qua SMTP (Mailtrap, Gmail, SES…)
/// ═══════════════════════════════════════════════════════════════════════
/// Đây là một **concrete implementation** của port `MailTransport`.
/// Chỉ biết cách gửi qua SMTP, không biết gì về business logic.
/// ═══════════════════════════════════════════════════════════════════════
pub struct SmtpTransport {
    mailer: AsyncSmtpTransport<Tokio1Executor>,
}

impl SmtpTransport {
    pub fn new(host: &str, port: u16, username: &str, password: &str) -> Result<Self, DomainError> {
        let creds = Credentials::new(username.to_string(), password.to_string());

        let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(host)
            .map_err(|e| DomainError::InfrastructureError(format!("SMTP config error: {}", e)))?
            .port(port)
            .credentials(creds)
            .build();

        info!("SMTP transport initialized: {}:{}", host, port);
        Ok(Self { mailer })
    }
}

fn to_mailbox(addr: &MailAddress) -> Result<Mailbox, DomainError> {
    let email_addr: lettre::Address = addr
        .email
        .parse()
        .map_err(|e| DomainError::ValidationError(format!("Invalid email '{}': {}", addr.email, e)))?;

    Ok(match &addr.name {
        Some(name) => Mailbox::new(Some(name.clone()), email_addr),
        None => Mailbox::new(None, email_addr),
    })
}

#[async_trait]
impl MailTransport for SmtpTransport {
    async fn send(&self, message: &MailMessage) -> Result<(), DomainError> {
        let from = to_mailbox(&message.from)?;

        for recipient in &message.to {
            let to = to_mailbox(recipient)?;

            let email_builder = Message::builder()
                .from(from.clone())
                .to(to)
                .subject(&message.subject);

            let email = match (&message.body.text, &message.body.html) {
                // Cả text và HTML → multipart/alternative
                (Some(text), Some(html)) => email_builder
                    .multipart(
                        MultiPart::alternative()
                            .singlepart(
                                SinglePart::builder()
                                    .header(ContentType::TEXT_PLAIN)
                                    .body(text.clone()),
                            )
                            .singlepart(
                                SinglePart::builder()
                                    .header(ContentType::TEXT_HTML)
                                    .body(html.clone()),
                            ),
                    )
                    .map_err(|e| {
                        DomainError::InfrastructureError(format!("Build email error: {}", e))
                    })?,
                // Chỉ HTML
                (None, Some(html)) => email_builder
                    .header(ContentType::TEXT_HTML)
                    .body(html.clone())
                    .map_err(|e| {
                        DomainError::InfrastructureError(format!("Build email error: {}", e))
                    })?,
                // Chỉ text
                (Some(text), None) => email_builder
                    .header(ContentType::TEXT_PLAIN)
                    .body(text.clone())
                    .map_err(|e| {
                        DomainError::InfrastructureError(format!("Build email error: {}", e))
                    })?,
                // Không có body (đã validate ở trên, nhưng guard thêm)
                (None, None) => {
                    return Err(DomainError::ValidationError("Email body is empty".into()));
                }
            };

            self.mailer.send(email).await.map_err(|e| {
                DomainError::InfrastructureError(format!(
                    "SMTP send failed to {}: {}",
                    recipient.email, e
                ))
            })?;
        }

        Ok(())
    }
}
