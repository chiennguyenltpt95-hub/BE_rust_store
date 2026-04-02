-- Cross-service FK hardening (safe rollout)
-- Add FK carts.user_id -> users.id as NOT VALID to avoid blocking old data.

DO $$
BEGIN
    IF to_regclass('public.carts') IS NOT NULL
       AND to_regclass('public.users') IS NOT NULL
       AND NOT EXISTS (
           SELECT 1 FROM pg_constraint WHERE conname = 'fk_carts_user_id_users'
       ) THEN
        ALTER TABLE carts
        ADD CONSTRAINT fk_carts_user_id_users
        FOREIGN KEY (user_id) REFERENCES users(id)
        ON DELETE RESTRICT
        NOT VALID;
    END IF;
END $$;
