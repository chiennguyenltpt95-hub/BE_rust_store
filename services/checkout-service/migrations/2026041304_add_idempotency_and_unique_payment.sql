ALTER TABLE checkouts
ADD COLUMN IF NOT EXISTS idempotency_key VARCHAR(128);

CREATE UNIQUE INDEX IF NOT EXISTS uq_checkouts_idempotency_key
ON checkouts(idempotency_key)
WHERE idempotency_key IS NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS uq_payment_transactions_provider_payment
ON payment_transactions(provider, provider_payment_id);
