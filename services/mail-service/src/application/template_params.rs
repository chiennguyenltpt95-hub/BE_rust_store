pub trait TemplateContextBuilder {
    fn build_context(&self) -> serde_json::Value;
}

#[derive(Debug, Clone)]
pub struct WelcomeTemplateParams {
    full_name: String,
    email: String,
    verify_url: String,
    cta_text: String,
    product_name: String,
    support_email: String,
}

impl WelcomeTemplateParams {
    pub fn builder() -> WelcomeTemplateParamsBuilder {
        WelcomeTemplateParamsBuilder {
            params: Self {
                full_name: "there".to_string(),
                email: String::new(),
                verify_url: "#".to_string(),
                cta_text: "Verify your account".to_string(),
                product_name: "Store Platform".to_string(),
                support_email: "support@store.local".to_string(),
            },
        }
    }
}

impl TemplateContextBuilder for WelcomeTemplateParams {
    fn build_context(&self) -> serde_json::Value {
        serde_json::json!({
            "full_name": self.full_name,
            "email": self.email,
            "verify_url": self.verify_url,
            "cta_text": self.cta_text,
            "product_name": self.product_name,
            "support_email": self.support_email,
        })
    }
}

pub struct WelcomeTemplateParamsBuilder {
    params: WelcomeTemplateParams,
}

impl WelcomeTemplateParamsBuilder {
    pub fn full_name(mut self, value: impl Into<String>) -> Self {
        self.params.full_name = value.into();
        self
    }

    pub fn email(mut self, value: impl Into<String>) -> Self {
        self.params.email = value.into();
        self
    }

    pub fn verify_url(mut self, value: impl Into<String>) -> Self {
        self.params.verify_url = value.into();
        self
    }

    pub fn cta_text(mut self, value: impl Into<String>) -> Self {
        self.params.cta_text = value.into();
        self
    }

    pub fn product_name(mut self, value: impl Into<String>) -> Self {
        self.params.product_name = value.into();
        self
    }

    pub fn support_email(mut self, value: impl Into<String>) -> Self {
        self.params.support_email = value.into();
        self
    }

    pub fn build(self) -> WelcomeTemplateParams {
        self.params
    }
}
