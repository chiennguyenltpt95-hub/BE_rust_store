use crate::application::commands::SendTemplatedMailCommand;
use crate::application::template_params::{TemplateContextBuilder, WelcomeTemplateParams};

#[derive(Debug, Clone)]
pub struct WelcomeMailFactoryConfig {
    pub verify_base_url: String,
    pub cta_text: String,
    pub product_name: String,
    pub support_email: String,
    pub subject: String,
}

#[derive(Debug, Clone)]
pub struct UserCreatedPayload {
    pub email: String,
    pub full_name: String,
    pub token_verify: String,
}

#[derive(Debug, Clone)]
pub struct WelcomeMailCommandFactory {
    cfg: WelcomeMailFactoryConfig,
}

impl WelcomeMailCommandFactory {
    pub fn new(cfg: WelcomeMailFactoryConfig) -> Self {
        Self { cfg }
    }

    pub fn create(&self, payload: &UserCreatedPayload) -> SendTemplatedMailCommand {
        let verify_url = if payload.token_verify.is_empty() {
            "#".to_string()
        } else {
            format!(
                "{}/{}",
                self.cfg.verify_base_url.trim_end_matches('/'),
                payload.token_verify
            )
        };

        let context = WelcomeTemplateParams::builder()
            .full_name(payload.full_name.clone())
            .email(payload.email.clone())
            .verify_url(verify_url)
            .cta_text(self.cfg.cta_text.clone())
            .product_name(self.cfg.product_name.clone())
            .support_email(self.cfg.support_email.clone())
            .build()
            .build_context();

        SendTemplatedMailCommand {
            to: payload.email.clone(),
            to_name: Some(payload.full_name.clone()),
            template_name: "welcome.html".to_string(),
            subject: self.cfg.subject.clone(),
            context,
        }
    }
}
