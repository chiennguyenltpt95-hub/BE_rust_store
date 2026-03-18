use async_trait::async_trait;
use domain_core::error::DomainError;

use super::models::MailMessage;

/// ═══════════════════════════════════════════════════════════════════════
/// PORT: MailTransport (Strategy Pattern)
/// ═══════════════════════════════════════════════════════════════════════
/// Đây là **trait trừu tượng** (port) định nghĩa hành vi "gửi mail".
/// Mọi nhà cung cấp mail (SMTP, SendGrid, AWS SES, Mailgun…)
/// chỉ cần implement trait này.
///
/// → Khi muốn đổi provider, bạn chỉ cần viết thêm 1 struct mới
///   implement `MailTransport`, KHÔNG cần sửa bất kỳ code nào ở
///   domain hay application layer.
/// ═══════════════════════════════════════════════════════════════════════
#[async_trait]
pub trait MailTransport: Send + Sync {
    /// Gửi một email. Trả về `Ok(())` nếu thành công.
    async fn send(&self, message: &MailMessage) -> Result<(), DomainError>;
}

/// ═══════════════════════════════════════════════════════════════════════
/// PORT: TemplateEngine (Strategy Pattern)
/// ═══════════════════════════════════════════════════════════════════════
/// Trait trừu tượng cho việc render template email.
/// Có thể đổi từ Tera sang Handlebars, MiniJinja… mà không sửa logic.
/// ═══════════════════════════════════════════════════════════════════════
#[async_trait]
pub trait TemplateEngine: Send + Sync {
    /// Render template với tên và dữ liệu truyền vào.
    fn render(
        &self,
        template_name: &str,
        context: &serde_json::Value,
    ) -> Result<String, DomainError>;
}
