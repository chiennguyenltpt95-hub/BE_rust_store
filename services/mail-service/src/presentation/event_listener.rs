use std::sync::Arc;

use prost::Message;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::message::Message as KafkaMessage;
use tracing::{error, info, warn};

use crate::application::commands::SendTemplatedMailCommand;
use crate::application::services::MailAppService;
use crate::application::template_factory::{UserCreatedPayload, WelcomeMailCommandFactory};

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
    welcome_factory: Arc<WelcomeMailCommandFactory>,
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

    info!(
        "Kafka event listener started (protobuf) — consuming topic '{}'",
        topic
    );

    use futures::StreamExt;
    let mut stream = consumer.stream();

    while let Some(result) = stream.next().await {
        match result {
            Ok(msg) => {
                if let Some(payload) = msg.payload() {
                    match proto::DomainEventEnvelope::decode(payload) {
                        Ok(envelope) => {
                            info!(event_type = %envelope.event_type, "Received protobuf event");

                            match envelope.payload {
                                Some(proto::domain_event_envelope::Payload::UserCreated(event)) => {
                                    handle_user_created(&mail_svc, &welcome_factory, &event).await;
                                }
                                Some(
                                    proto::domain_event_envelope::Payload::UserVerificationRequested(
                                        event,
                                    ),
                                ) => {
                                    handle_user_verification_requested(
                                        &mail_svc,
                                        &welcome_factory,
                                        &event,
                                    )
                                    .await;
                                }
                                Some(proto::domain_event_envelope::Payload::OrderConfirmedMail(event)) => {
                                    handle_order_confirmed_mail(&mail_svc, &event).await;
                                }
                                _ => {}
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

async fn handle_order_confirmed_mail(
    mail_svc: &MailAppService,
    event: &proto::OrderConfirmedMailEvent,
) {
    if event.to.is_empty() {
        warn!("order.confirmed.mail event missing recipient");
        return;
    }

    let context: serde_json::Value =
        serde_json::from_str(&event.context_json).unwrap_or_else(|_| serde_json::json!({}));

    let cmd = SendTemplatedMailCommand {
        to: event.to.clone(),
        to_name: if event.to_name.is_empty() {
            None
        } else {
            Some(event.to_name.clone())
        },
        template_name: event.template_name.clone(),
        subject: event.subject.clone(),
        context,
    };

    if let Err(e) = mail_svc.send_templated_mail(cmd).await {
        error!(
            "Failed to send order confirmation email to {}: {}",
            event.to, e
        );
    } else {
        info!("Order confirmation email sent to {}", event.to);
    }
}

/// Xử lý event user.verify_requested → gửi lại email verify
async fn handle_user_verification_requested(
    mail_svc: &MailAppService,
    welcome_factory: &WelcomeMailCommandFactory,
    event: &proto::UserVerificationRequestedEvent,
) {
    if event.email.is_empty() {
        warn!("user.verify_requested event missing email field");
        return;
    }

    let payload = UserCreatedPayload {
        email: event.email.clone(),
        full_name: event.full_name.clone(),
        token_verify: event.token_verify.clone(),
    };

    let cmd = welcome_factory.create(&payload);

    if let Err(e) = mail_svc.send_templated_mail(cmd).await {
        error!("Failed to resend verify email to {}: {}", event.email, e);
    } else {
        info!("Verification email resent to {}", event.email);
    }
}

/// Xử lý event user.created → gửi welcome email
async fn handle_user_created(
    mail_svc: &MailAppService,
    welcome_factory: &WelcomeMailCommandFactory,
    event: &proto::UserCreatedEvent,
) {
    if event.email.is_empty() {
        warn!("user.created event missing email field");
        return;
    }

    let payload = UserCreatedPayload {
        email: event.email.clone(),
        full_name: event.full_name.clone(),
        token_verify: event.token_verify.clone(),
    };

    let cmd = welcome_factory.create(&payload);

    if let Err(e) = mail_svc.send_templated_mail(cmd).await {
        error!("Failed to send welcome email to {}: {}", event.email, e);
    } else {
        info!("Welcome email sent to {}", event.email);
    }
}
