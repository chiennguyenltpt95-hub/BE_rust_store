use async_trait::async_trait;
use domain_core::error::DomainError;
use tracing::info;

use crate::domain::models::MailMessage;
use crate::domain::ports::MailTransport;

/// ═══════════════════════════════════════════════════════════════════════
/// ADAPTER: ConsoleTransport — in email ra console (dùng cho dev/test)
/// ═══════════════════════════════════════════════════════════════════════
/// Khi phát triển local, không cần kết nối SMTP thật.
/// Chỉ cần set MAIL_TRANSPORT=console trong .env.
/// ═══════════════════════════════════════════════════════════════════════
pub struct ConsoleTransport;

#[async_trait]
impl MailTransport for ConsoleTransport {
    async fn send(&self, message: &MailMessage) -> Result<(), DomainError> {
        info!("═══════════════ CONSOLE MAIL ═══════════════");
        info!("From: {} <{}>",
            message.from.name.as_deref().unwrap_or(""),
            message.from.email
        );
        for to in &message.to {
            info!("To: {} <{}>",
                to.name.as_deref().unwrap_or(""),
                to.email
            );
        }
        info!("Subject: {}", message.subject);
        if let Some(text) = &message.body.text {
            info!("Body (text):\n{}", text);
        }
        if let Some(html) = &message.body.html {
            info!("Body (html):\n{}", html);
        }
        info!("═══════════════════════════════════════════");
        Ok(())
    }
}
