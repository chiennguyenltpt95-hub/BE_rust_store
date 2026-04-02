use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

#[async_trait]
pub trait NotificationChannel: Send + Sync {
    async fn send(
        &self,
        recipient: &str,
        template_name: Option<&str>,
        payload: &serde_json::Value,
    ) -> Result<(), String>;
}

pub struct NoopChannel {
    name: &'static str,
}

impl NoopChannel {
    pub fn new(name: &'static str) -> Self {
        Self { name }
    }
}

#[async_trait]
impl NotificationChannel for NoopChannel {
    async fn send(
        &self,
        recipient: &str,
        template_name: Option<&str>,
        payload: &serde_json::Value,
    ) -> Result<(), String> {
        tracing::info!(
            channel = %self.name,
            recipient = %recipient,
            template_name = ?template_name,
            payload = %payload,
            "Noop channel accepted notification"
        );
        Ok(())
    }
}

pub struct TelegramChannel {
    client: reqwest::Client,
    bot_token: String,
}

impl TelegramChannel {
    pub fn new(bot_token: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            bot_token,
        }
    }

    fn build_message(
        template_name: Option<&str>,
        payload: &serde_json::Value,
        recipient: &str,
    ) -> String {
        if let Some(text) = payload.get("text").and_then(|v| v.as_str()) {
            return text.to_string();
        }

        let checkout_id = payload
            .get("checkout_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let amount_cents = payload
            .get("amount_cents")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let currency = payload
            .get("currency")
            .and_then(|v| v.as_str())
            .unwrap_or("N/A");
        let checkout_url = payload
            .get("checkout_url")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        format!(
            "[{}] New checkout created\\nrecipient: {}\\ncheckout_id: {}\\namount: {} {}\\nurl: {}",
            template_name.unwrap_or("checkout_created"),
            recipient,
            checkout_id,
            amount_cents,
            currency,
            checkout_url
        )
    }
}

#[async_trait]
impl NotificationChannel for TelegramChannel {
    async fn send(
        &self,
        recipient: &str,
        template_name: Option<&str>,
        payload: &serde_json::Value,
    ) -> Result<(), String> {
        if self.bot_token.trim().is_empty() {
            return Err("TELEGRAM_BOT_TOKEN is empty".to_string());
        }

        let text = Self::build_message(template_name, payload, recipient);
        let endpoint = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token);

        let resp = self
            .client
            .post(endpoint)
            .json(&serde_json::json!({
                "chat_id": recipient,
                "text": text,
                "disable_web_page_preview": false
            }))
            .send()
            .await
            .map_err(|e| format!("Telegram request failed: {}", e))?;

        if !resp.status().is_success() {
            let body = resp
                .text()
                .await
                .unwrap_or_else(|_| "<no response body>".to_string());
            return Err(format!("Telegram API returned non-success: {}", body));
        }

        Ok(())
    }
}

pub struct ChannelFactory {
    channels: HashMap<String, Arc<dyn NotificationChannel>>,
}

impl ChannelFactory {
    pub fn new(telegram_bot_token: String) -> Self {
        let mut channels: HashMap<String, Arc<dyn NotificationChannel>> = HashMap::new();

        channels.insert("telegram".to_string(), Arc::new(TelegramChannel::new(telegram_bot_token)));
        channels.insert("email".to_string(), Arc::new(NoopChannel::new("email")));
        channels.insert("sms".to_string(), Arc::new(NoopChannel::new("sms")));
        channels.insert("push".to_string(), Arc::new(NoopChannel::new("push")));
        channels.insert("webhook".to_string(), Arc::new(NoopChannel::new("webhook")));

        Self { channels }
    }

    pub fn get(&self, channel: &str) -> Option<Arc<dyn NotificationChannel>> {
        self.channels.get(channel).cloned()
    }
}
