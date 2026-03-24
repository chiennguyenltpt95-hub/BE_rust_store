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
    /// Khởi tạo template engine.
    /// Ưu tiên load template từ folder HTML để chỉnh sửa linh hoạt,
    /// fallback sang builtin nếu không tìm thấy file ngoài.
    pub fn new() -> Result<Self, DomainError> {
        for glob in [
            "services/mail-service/templates/html/**/*.html",
            "templates/html/**/*.html",
        ] {
            if let Ok(tera) = Tera::new(glob) {
                if tera.get_template_names().next().is_some() {
                    return Ok(Self { tera });
                }
            }
        }

        let mut tera = Tera::default();

        // ── Template: Welcome ────────────────────────────────────────
        tera.add_raw_template("welcome", include_str!("builtin/welcome.html"))
            .map_err(|e| DomainError::InfrastructureError(format!("Template error: {}", e)))?;
        tera.add_raw_template("welcome.html", include_str!("builtin/welcome.html"))
            .map_err(|e| DomainError::InfrastructureError(format!("Template error: {}", e)))?;

        // ── Template: Reset Password ─────────────────────────────────
        tera.add_raw_template(
            "reset_password",
            include_str!("builtin/reset_password.html"),
        )
        .map_err(|e| DomainError::InfrastructureError(format!("Template error: {}", e)))?;
        tera.add_raw_template(
            "reset_password.html",
            include_str!("builtin/reset_password.html"),
        )
        .map_err(|e| DomainError::InfrastructureError(format!("Template error: {}", e)))?;

        // ── Template: Order Confirmation ─────────────────────────────
        tera.add_raw_template(
            "order_confirmation",
            include_str!("builtin/order_confirmation.html"),
        )
        .map_err(|e| DomainError::InfrastructureError(format!("Template error: {}", e)))?;
        tera.add_raw_template(
            "order_confirmation.html",
            include_str!("builtin/order_confirmation.html"),
        )
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
        let ctx = Context::from_value(context.clone()).map_err(|e| {
            DomainError::ValidationError(format!("Invalid template context: {}", e))
        })?;

        let mut candidates = vec![template_name.to_string()];
        candidates.push(format!("templates/html/{}", template_name));
        candidates.push(format!("html/{}", template_name));
        if let Some(without_ext) = template_name.strip_suffix(".html") {
            candidates.push(without_ext.to_string());
            candidates.push(format!("templates/html/{}", without_ext));
            candidates.push(format!("html/{}", without_ext));
        }

        let mut last_missing_err: Option<String> = None;
        for name in candidates {
            match self.tera.render(&name, &ctx) {
                Ok(rendered) => return Ok(rendered),
                Err(err) => {
                    let msg = err.to_string();
                    if msg.contains("not found") || msg.contains("Template '") {
                        last_missing_err = Some(format!("{}: {}", name, msg));
                        continue;
                    }

                    return Err(DomainError::InfrastructureError(format!(
                        "Render '{}' failed on '{}': {}",
                        template_name, name, msg
                    )));
                }
            }
        }

        Err(DomainError::InfrastructureError(format!(
            "Render '{}' failed: {}",
            template_name,
            last_missing_err.unwrap_or_else(|| "template not found in known paths".to_string())
        )))
    }
}
