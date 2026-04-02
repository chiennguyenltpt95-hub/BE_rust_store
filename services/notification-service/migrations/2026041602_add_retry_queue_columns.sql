-- Add async queue/retry support for notifications

ALTER TABLE notifications
ADD COLUMN IF NOT EXISTS attempts INTEGER NOT NULL DEFAULT 0,
ADD COLUMN IF NOT EXISTS max_attempts INTEGER NOT NULL DEFAULT 5,
ADD COLUMN IF NOT EXISTS next_retry_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
ADD COLUMN IF NOT EXISTS last_error TEXT,
ADD COLUMN IF NOT EXISTS processed_at TIMESTAMPTZ,
ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

-- Re-create checks to include telegram channel and processing status.
ALTER TABLE notifications DROP CONSTRAINT IF EXISTS notifications_channel_check;
ALTER TABLE notifications DROP CONSTRAINT IF EXISTS notifications_status_check;

ALTER TABLE notifications
ADD CONSTRAINT notifications_channel_check
CHECK (channel IN ('telegram', 'email', 'sms', 'push', 'webhook'));

ALTER TABLE notifications
ADD CONSTRAINT notifications_status_check
CHECK (status IN ('queued', 'processing', 'sent', 'failed'));

CREATE INDEX IF NOT EXISTS idx_notifications_queue
ON notifications(status, next_retry_at, created_at);
