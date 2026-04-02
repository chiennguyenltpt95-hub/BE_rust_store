-- Harden users table constraints/indexes for data quality

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'chk_users_age_range'
    ) THEN
        ALTER TABLE users
        ADD CONSTRAINT chk_users_age_range
        CHECK (age IS NULL OR (age >= 0 AND age <= 150));
    END IF;
END $$;

CREATE UNIQUE INDEX IF NOT EXISTS uq_users_wallet_address
ON users(wallet_address)
WHERE wallet_address IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_users_verified
ON users(verified);
