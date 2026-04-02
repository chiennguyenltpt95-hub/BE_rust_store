CREATE TABLE IF NOT EXISTS checkouts (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    cart_id UUID NOT NULL,
    amount_cents BIGINT NOT NULL CHECK (amount_cents >= 0),
    currency VARCHAR(16) NOT NULL,
    status VARCHAR(20) NOT NULL CHECK (status IN ('pending', 'paid', 'failed', 'cancelled')),
    payment_method VARCHAR(20) NOT NULL CHECK (payment_method IN ('paypal', 'stripe', 'oxapay')),
    external_payment_id VARCHAR(255),
    checkout_url TEXT,
    failure_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_checkouts_user_id ON checkouts(user_id);
CREATE INDEX IF NOT EXISTS idx_checkouts_cart_id ON checkouts(cart_id);
CREATE INDEX IF NOT EXISTS idx_checkouts_status ON checkouts(status);
