use std::sync::Arc;

use prost::Message;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::message::Message as KafkaMessage;
use tracing::{error, info, warn};

use crate::application::commands::SendTemplatedMailCommand;
use crate::application::services::MailAppService;

/// Generated protobuf types
pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/store.events.rs"));
}

/// Lắng nghe domain events từ Kafka và gửi mail tương ứng.
pub async fn start_event_listener(
    brokers: &str,
    topic: &str,
    group_id: &str,
    mail_svc: Arc<MailAppService>,
) -> anyhow::Result<()> {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("group.id", group_id)
        .set("enable.auto.commit", "false")
        .set("auto.offset.reset", "earliest")
        .set("session.timeout.ms", "10000")
        .create()
        .map_err(|e| anyhow::anyhow!("Failed to create Kafka consumer: {}", e))?;

    consumer
        .subscribe(&[topic])
        .map_err(|e| anyhow::anyhow!("Failed to subscribe to topic '{}': {}", topic, e))?;

    info!("Kafka event listener started (protobuf) — consuming topic '{}'", topic);

    use futures::StreamExt;
    let mut stream = consumer.stream();

    while let Some(result) = stream.next().await {
        match result {
            Ok(msg) => {
                if let Some(payload) = msg.payload() {
                    match proto::DomainEventEnvelope::decode(payload) {
                        Ok(envelope) => {
                            info!(event_type = %envelope.event_type, "Received protobuf event");

                            if let Some(proto::domain_event_envelope::Payload::UserCreated(event)) =
                                envelope.payload
                            {
                                handle_user_created(&mail_svc, &event).await;
                            }
                        }
                        Err(e) => {
                            warn!("Failed to decode protobuf event: {}", e);
                        }
                    }
                }

                if let Err(e) = consumer.commit_message(&msg, CommitMode::Async) {
                    error!("Commit failed: {}", e);
                }
            }
            Err(e) => {
                warn!("Kafka consumer error: {}", e);
            }
        }
    }

    Ok(())
}

/// Xử lý event user.created → gửi welcome email
async fn handle_user_created(mail_svc: &MailAppService, event: &proto::UserCreatedEvent) {
    if event.email.is_empty() {
        warn!("user.created event missing email field");
        return;
    }

    let cmd = SendTemplatedMailCommand {
        to: event.email.clone(),
        to_name: Some(event.full_name.clone()),
        template_name: "welcome".to_string(),
        subject: "Chào mừng bạn đến với Store! 🎉".to_string(),
        context: serde_json::json!({
            "full_name": event.full_name,
            "email": event.email,
        }),
    };

    if let Err(e) = mail_svc.send_templated_mail(cmd).await {
        error!("Failed to send welcome email to {}: {}", event.email, e);
    } else {
        info!("Welcome email sent to {}", event.email);
    }
}
