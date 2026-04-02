ALTER TABLE outbox_messages
DROP CONSTRAINT IF EXISTS outbox_messages_status_check;

ALTER TABLE outbox_messages
ADD CONSTRAINT outbox_messages_status_check
CHECK (status IN ('pending', 'processing', 'sent', 'failed', 'dead_letter'));
