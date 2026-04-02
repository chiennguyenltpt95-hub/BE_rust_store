use std::sync::Arc;

use crate::application::channels::ChannelFactory;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
struct QueuedNotification {
    id: Uuid,
    channel: String,
    recipient: String,
    template_name: Option<String>,
    payload: serde_json::Value,
    attempts: i32,
    max_attempts: i32,
}

pub async fn process_once(
    pool: &PgPool,
    channel_factory: Arc<ChannelFactory>,
    batch_size: i64,
) -> Result<usize, sqlx::Error> {
    let rows: Vec<QueuedNotification> = sqlx::query_as(
        r#"
        WITH picked AS (
            SELECT id
            FROM notifications
            WHERE status = 'queued'
              AND next_retry_at <= NOW()
            ORDER BY created_at ASC
            LIMIT $1
            FOR UPDATE SKIP LOCKED
        )
        UPDATE notifications n
        SET status = 'processing',
            updated_at = NOW()
        WHERE n.id IN (SELECT id FROM picked)
        RETURNING n.id, n.channel, n.recipient, n.template_name, n.payload, n.attempts, n.max_attempts
        "#,
    )
    .bind(batch_size)
    .fetch_all(pool)
    .await?;

    let mut processed = 0usize;

    for row in rows {
        processed += 1;
        let Some(channel) = channel_factory.get(&row.channel) else {
            mark_failed_terminal(pool, row.id, row.attempts, "Unsupported channel").await?;
            continue;
        };

        match channel
            .send(&row.recipient, row.template_name.as_deref(), &row.payload)
            .await
        {
            Ok(_) => {
                sqlx::query(
                    r#"UPDATE notifications
                       SET status = 'sent',
                           processed_at = NOW(),
                           last_error = NULL,
                           updated_at = NOW()
                       WHERE id = $1"#,
                )
                .bind(row.id)
                .execute(pool)
                .await?;
            }
            Err(err) => {
                let next_attempt = row.attempts + 1;
                if next_attempt >= row.max_attempts {
                    mark_failed_terminal(pool, row.id, next_attempt, &err).await?;
                } else {
                    let delay_secs = backoff_seconds(next_attempt);
                    sqlx::query(
                        r#"UPDATE notifications
                           SET status = 'queued',
                               attempts = $2,
                               last_error = $3,
                               next_retry_at = NOW() + ($4::text || ' seconds')::interval,
                               updated_at = NOW()
                           WHERE id = $1"#,
                    )
                    .bind(row.id)
                    .bind(next_attempt)
                    .bind(err)
                    .bind(delay_secs)
                    .execute(pool)
                    .await?;
                }
            }
        }
    }

    Ok(processed)
}

async fn mark_failed_terminal(
    pool: &PgPool,
    id: Uuid,
    attempts: i32,
    error: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"UPDATE notifications
           SET status = 'failed',
               attempts = $2,
               last_error = $3,
               processed_at = NOW(),
               updated_at = NOW()
           WHERE id = $1"#,
    )
    .bind(id)
    .bind(attempts)
    .bind(error)
    .execute(pool)
    .await?;

    Ok(())
}

fn backoff_seconds(attempt: i32) -> i64 {
    match attempt {
        1 => 5,
        2 => 15,
        3 => 30,
        4 => 60,
        _ => 120,
    }
}
