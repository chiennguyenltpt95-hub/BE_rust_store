use prost::Message;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::message::Message as KafkaMessage;
use sqlx::PgPool;
use tracing::{error, info, warn};
use uuid::Uuid;

pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/store.events.rs"));
}

pub async fn start_event_listener(
    brokers: &str,
    topic: &str,
    group_id: &str,
    pool: PgPool,
    default_max_attempts: i32,
    default_telegram_recipient: String,
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

    info!("Notification Kafka listener started on topic '{}'", topic);

    use futures::StreamExt;
    let mut stream = consumer.stream();

    while let Some(result) = stream.next().await {
        match result {
            Ok(msg) => {
                if let Some(payload) = msg.payload() {
                    if let Err(err) = handle_message(
                        payload,
                        &pool,
                        default_max_attempts,
                        &default_telegram_recipient,
                    )
                    .await
                    {
                        warn!("Failed to handle Kafka notification event: {}", err);
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

async fn handle_message(
    payload: &[u8],
    pool: &PgPool,
    default_max_attempts: i32,
    default_telegram_recipient: &str,
) -> anyhow::Result<()> {
    let envelope = proto::DomainEventEnvelope::decode(payload)?;

    if envelope.event_type != "checkout.created.notification" {
        return Ok(());
    }

    let Some(proto::domain_event_envelope::Payload::CheckoutCreatedNotification(event)) =
        envelope.payload
    else {
        return Ok(());
    };

    let payload_json: serde_json::Value = serde_json::from_str(&event.payload_json)
        .unwrap_or_else(|_| serde_json::json!({"text": event.payload_json}));

    let max_attempts = if event.max_attempts > 0 {
        event.max_attempts
    } else {
        default_max_attempts.max(1)
    };

    let recipient = if event.recipient.trim().is_empty() {
        default_telegram_recipient.to_string()
    } else {
        event.recipient.clone()
    };

    if recipient.trim().is_empty() {
        return Ok(());
    }

    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO notifications
           (id, channel, recipient, template_name, payload, status, attempts, max_attempts, next_retry_at, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, 'queued', 0, $6, NOW(), NOW(), NOW())"#,
    )
    .bind(id)
    .bind(event.channel.to_lowercase())
    .bind(recipient)
    .bind(Some(event.template_name))
    .bind(payload_json)
    .bind(max_attempts)
    .execute(pool)
    .await?;

    Ok(())
}
