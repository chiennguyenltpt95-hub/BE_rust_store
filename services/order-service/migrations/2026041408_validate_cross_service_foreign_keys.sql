-- Validate cross-service FK constraints after cleaning legacy bad rows.
-- This migration is idempotent and safe to run multiple times.

DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'fk_carts_user_id_users' AND convalidated = false
    ) THEN
        ALTER TABLE carts VALIDATE CONSTRAINT fk_carts_user_id_users;
    END IF;

    IF EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'fk_checkouts_user_id_users' AND convalidated = false
    ) THEN
        ALTER TABLE checkouts VALIDATE CONSTRAINT fk_checkouts_user_id_users;
    END IF;

    IF EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'fk_checkouts_cart_id_carts' AND convalidated = false
    ) THEN
        ALTER TABLE checkouts VALIDATE CONSTRAINT fk_checkouts_cart_id_carts;
    END IF;

    IF EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'fk_orders_user_id_users' AND convalidated = false
    ) THEN
        ALTER TABLE orders VALIDATE CONSTRAINT fk_orders_user_id_users;
    END IF;

    IF EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'fk_orders_checkout_id_checkouts' AND convalidated = false
    ) THEN
        ALTER TABLE orders VALIDATE CONSTRAINT fk_orders_checkout_id_checkouts;
    END IF;

    IF EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'fk_orders_cart_id_carts' AND convalidated = false
    ) THEN
        ALTER TABLE orders VALIDATE CONSTRAINT fk_orders_cart_id_carts;
    END IF;
END $$;
