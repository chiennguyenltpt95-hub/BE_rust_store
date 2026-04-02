use domain_core::error::DomainError;

use crate::application::ports::{PaymentGateway, PaymentGatewayFactoryPort};
use crate::config::AppConfig;
use crate::domain::entities::checkout::PaymentMethod;

use super::oxapay_gateway::OxaPayGateway;
use super::paypal_gateway::PaypalGateway;
use super::stripe_gateway::StripeGateway;

pub struct PaymentGatewayFactory {
    paypal: PaypalGateway,
    stripe: StripeGateway,
    oxapay: OxaPayGateway,
}

impl PaymentGatewayFactory {
    pub fn new(cfg: AppConfig) -> Self {
        Self {
            paypal: PaypalGateway::new(
                cfg.paypal_api_base_url,
                cfg.paypal_api_key,
                cfg.payment_sandbox_mode,
            ),
            stripe: StripeGateway::new(
                cfg.stripe_api_base_url,
                cfg.stripe_api_key,
                cfg.payment_sandbox_mode,
            ),
            oxapay: OxaPayGateway::new(
                cfg.oxapay_api_base_url,
                cfg.oxapay_api_key,
                cfg.payment_sandbox_mode,
            ),
        }
    }
}

impl PaymentGatewayFactoryPort for PaymentGatewayFactory {
    fn get_gateway(&self, method: &PaymentMethod) -> Result<&dyn PaymentGateway, DomainError> {
        let gateway: &dyn PaymentGateway = match method {
            PaymentMethod::Paypal => &self.paypal,
            PaymentMethod::Stripe => &self.stripe,
            PaymentMethod::OxaPay => &self.oxapay,
        };
        Ok(gateway)
    }
}
