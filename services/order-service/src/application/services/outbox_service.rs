use chrono::{Duration, Utc};
use domain_core::error::DomainError;
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::domain::entities::{OutboxMessage, OutboxStats};

use crate::application::ports::MailMessage;
use crate::domain::repositories::OrderRepository;
use crate::infrastructure::messaging::KafkaEventPublisher;

pub struct OutboxService {
    repo: Arc<dyn OrderRepository>,
    event_publisher: Arc<KafkaEventPublisher>,
    max_attempts: i32,
}

impl OutboxService {
    pub fn new(
        repo: Arc<dyn OrderRepository>,
        event_publisher: Arc<KafkaEventPublisher>,
        max_attempts: i32,
    ) -> Self {
        Self {
            repo,
            event_publisher,
            max_attempts,
        }
    }

    pub async fn process_once(&self, batch_size: i64) -> Result<usize, DomainError> {
        let messages = self.repo.dequeue_outbox_pending(batch_size).await?;
        let mut processed = 0usize;

        for msg in messages {
            info!(
                outbox_id = %msg.id,
                attempts = msg.attempts,
                event_type = %msg.event_type,
                "Processing outbox message"
            );

            let payload: Result<MailMessage, _> = serde_json::from_value(msg.payload.clone());
            match payload {
                Ok(mail) => match self
                    .event_publisher
                    .publish_order_confirmed_mail(&msg.aggregate_id.to_string(), &mail)
                    .await
                {
                    Ok(_) => {
                        self.repo.mark_outbox_sent(msg.id).await?;
                        info!(outbox_id = %msg.id, "Outbox message marked as sent");
                        processed += 1;
                    }
                    Err(err) => {
                        self.handle_failure(&msg, &err.to_string()).await?;
                    }
                },
                Err(err) => {
                    self.handle_failure(&msg, &format!("Invalid outbox payload: {}", err))
                        .await?;
                }
            }
        }

        Ok(processed)
    }

    pub async fn stats(&self) -> Result<OutboxStats, DomainError> {
        self.repo.get_outbox_stats().await
    }

    pub async fn list_messages(
        &self,
        status: Option<&str>,
        limit: i64,
    ) -> Result<Vec<OutboxMessage>, DomainError> {
        self.repo.list_outbox_messages(status, limit).await
    }

    async fn handle_failure(
        &self,
        msg: &OutboxMessage,
        error_message: &str,
    ) -> Result<(), DomainError> {
        let next_attempts = msg.attempts + 1;
        if next_attempts >= self.max_attempts {
            warn!(
                outbox_id = %msg.id,
                attempts = next_attempts,
                max_attempts = self.max_attempts,
                error = %error_message,
                "Outbox message moved to dead-letter"
            );
            self.repo
                .mark_outbox_dead_letter(msg.id, error_message)
                .await?;
            return Ok(());
        }

        let delay_secs = backoff_seconds(next_attempts);
        let retry_at = Utc::now() + Duration::seconds(delay_secs as i64);
        error!(
            outbox_id = %msg.id,
            attempts = next_attempts,
            next_retry_at = %retry_at,
            error = %error_message,
            "Outbox message failed and will retry"
        );
        self.repo
            .mark_outbox_failed(msg.id, error_message, retry_at)
            .await
    }
}

fn backoff_seconds(attempts: i32) -> u64 {
    match attempts {
        0 | 1 => 10,
        2 => 30,
        3 => 60,
        4 => 300,
        _ => 900,
    }
}
