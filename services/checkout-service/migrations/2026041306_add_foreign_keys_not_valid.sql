-- Cross-service FK hardening (safe rollout)
-- Add FKs on checkouts with NOT VALID for backward-compatible migration.

DO $$
BEGIN
    IF to_regclass('public.checkouts') IS NOT NULL
       AND to_regclass('public.users') IS NOT NULL
       AND NOT EXISTS (
           SELECT 1 FROM pg_constraint WHERE conname = 'fk_checkouts_user_id_users'
       ) THEN
        ALTER TABLE checkouts
        ADD CONSTRAINT fk_checkouts_user_id_users
        FOREIGN KEY (user_id) REFERENCES users(id)
        ON DELETE RESTRICT
        NOT VALID;
    END IF;

    IF to_regclass('public.checkouts') IS NOT NULL
       AND to_regclass('public.carts') IS NOT NULL
       AND NOT EXISTS (
           SELECT 1 FROM pg_constraint WHERE conname = 'fk_checkouts_cart_id_carts'
       ) THEN
        ALTER TABLE checkouts
        ADD CONSTRAINT fk_checkouts_cart_id_carts
        FOREIGN KEY (cart_id) REFERENCES carts(id)
        ON DELETE RESTRICT
        NOT VALID;
    END IF;
END $$;
