CREATE TABLE IF NOT EXISTS notifications (
    id UUID PRIMARY KEY,
    channel VARCHAR(20) NOT NULL CHECK (channel IN ('email', 'sms', 'push', 'webhook')),
    recipient VARCHAR(255) NOT NULL,
    template_name VARCHAR(100),
    payload JSONB NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'queued' CHECK (status IN ('queued', 'sent', 'failed')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_notifications_status_created
ON notifications(status, created_at DESC);
