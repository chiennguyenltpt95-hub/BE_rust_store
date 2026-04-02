-- Add human-readable order number and optimize query paths

ALTER TABLE orders
ADD COLUMN IF NOT EXISTS order_number VARCHAR(32);

CREATE UNIQUE INDEX IF NOT EXISTS uq_orders_order_number
ON orders(order_number)
WHERE order_number IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_orders_created_at
ON orders(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_outbox_status_attempts
ON outbox_messages(status, attempts, next_retry_at);
