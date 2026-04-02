CREATE TABLE IF NOT EXISTS payment_transactions (
    id UUID PRIMARY KEY,
    checkout_id UUID NOT NULL REFERENCES checkouts(id) ON DELETE CASCADE,
    provider VARCHAR(20) NOT NULL,
    provider_payment_id VARCHAR(255) NOT NULL,
    amount_cents BIGINT NOT NULL CHECK (amount_cents >= 0),
    currency VARCHAR(16) NOT NULL,
    status VARCHAR(20) NOT NULL,
    raw_response JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_payment_transactions_checkout_id ON payment_transactions(checkout_id);
