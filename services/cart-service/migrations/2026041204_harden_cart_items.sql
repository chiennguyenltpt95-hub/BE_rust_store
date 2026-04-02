-- Improve cart_items consistency and retrieval performance

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'chk_cart_items_unit_price_non_negative'
    ) THEN
        ALTER TABLE cart_items
        ADD CONSTRAINT chk_cart_items_unit_price_non_negative
        CHECK (unit_price_cents >= 0);
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_cart_items_cart_created
ON cart_items(cart_id, created_at DESC);
