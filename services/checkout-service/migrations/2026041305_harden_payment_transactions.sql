-- Harden payment transaction semantics

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'chk_payment_transactions_provider'
    ) THEN
        ALTER TABLE payment_transactions
        ADD CONSTRAINT chk_payment_transactions_provider
        CHECK (provider IN ('paypal', 'stripe', 'oxapay'));
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_payment_transactions_provider_created
ON payment_transactions(provider, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_checkouts_status_updated
ON checkouts(status, updated_at DESC);
