-- Cross-service FK hardening (safe rollout)
-- Add FKs on orders with NOT VALID for phased enforcement.

DO $$
BEGIN
    IF to_regclass('public.orders') IS NOT NULL
       AND to_regclass('public.users') IS NOT NULL
       AND NOT EXISTS (
           SELECT 1 FROM pg_constraint WHERE conname = 'fk_orders_user_id_users'
       ) THEN
        ALTER TABLE orders
        ADD CONSTRAINT fk_orders_user_id_users
        FOREIGN KEY (user_id) REFERENCES users(id)
        ON DELETE RESTRICT
        NOT VALID;
    END IF;

    IF to_regclass('public.orders') IS NOT NULL
       AND to_regclass('public.checkouts') IS NOT NULL
       AND NOT EXISTS (
           SELECT 1 FROM pg_constraint WHERE conname = 'fk_orders_checkout_id_checkouts'
       ) THEN
        ALTER TABLE orders
        ADD CONSTRAINT fk_orders_checkout_id_checkouts
        FOREIGN KEY (checkout_id) REFERENCES checkouts(id)
        ON DELETE RESTRICT
        NOT VALID;
    END IF;

    IF to_regclass('public.orders') IS NOT NULL
       AND to_regclass('public.carts') IS NOT NULL
       AND NOT EXISTS (
           SELECT 1 FROM pg_constraint WHERE conname = 'fk_orders_cart_id_carts'
       ) THEN
        ALTER TABLE orders
        ADD CONSTRAINT fk_orders_cart_id_carts
        FOREIGN KEY (cart_id) REFERENCES carts(id)
        ON DELETE RESTRICT
        NOT VALID;
    END IF;
END $$;
