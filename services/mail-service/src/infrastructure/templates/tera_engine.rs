use async_trait::async_trait;
use domain_core::error::DomainError;
use tera::{Context, Tera};

use crate::domain::ports::TemplateEngine;

/// ═══════════════════════════════════════════════════════════════════════
/// ADAPTER: TeraTemplateEngine — render email template bằng Tera
/// ═══════════════════════════════════════════════════════════════════════
/// Tera là template engine (tương tự Jinja2 của Python).
/// Nếu sau này muốn đổi sang Handlebars hay MiniJinja,
/// chỉ cần implement TemplateEngine trait cho engine mới.
/// ═══════════════════════════════════════════════════════════════════════
pub struct TeraTemplateEngine {
    tera: Tera,
}

impl TeraTemplateEngine {
    /// Khởi tạo với các template mặc định (hardcoded).
    /// Production có thể load từ file: Tera::new("templates/**/*.html")
    pub fn new() -> Result<Self, DomainError> {
        let mut tera = Tera::default();

        // ── Template: Welcome ────────────────────────────────────────
        tera.add_raw_template("welcome", include_str!("builtin/welcome.html"))
            .map_err(|e| DomainError::InfrastructureError(format!("Template error: {}", e)))?;

        // ── Template: Reset Password ─────────────────────────────────
        tera.add_raw_template("reset_password", include_str!("builtin/reset_password.html"))
            .map_err(|e| DomainError::InfrastructureError(format!("Template error: {}", e)))?;

        // ── Template: Order Confirmation ─────────────────────────────
        tera.add_raw_template("order_confirmation", include_str!("builtin/order_confirmation.html"))
            .map_err(|e| DomainError::InfrastructureError(format!("Template error: {}", e)))?;

        Ok(Self { tera })
    }
}

#[async_trait]
impl TemplateEngine for TeraTemplateEngine {
    fn render(
        &self,
        template_name: &str,
        context: &serde_json::Value,
    ) -> Result<String, DomainError> {
        let ctx = Context::from_value(context.clone())
            .map_err(|e| DomainError::ValidationError(format!("Invalid template context: {}", e)))?;

        self.tera
            .render(template_name, &ctx)
            .map_err(|e| DomainError::InfrastructureError(format!("Render '{}' failed: {}", template_name, e)))
    }
}
