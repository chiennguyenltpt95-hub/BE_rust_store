CREATE TABLE IF NOT EXISTS product_comments (
    id UUID PRIMARY KEY,
    product_id UUID NOT NULL,
    user_id UUID NULL,
    content TEXT NOT NULL,
    rating INT NULL CHECK (rating >= 1 AND rating <= 5),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_product_comments_product_id
ON product_comments(product_id);

CREATE INDEX IF NOT EXISTS idx_product_comments_user_id
ON product_comments(user_id);

CREATE INDEX IF NOT EXISTS idx_product_comments_created_at
ON product_comments(created_at DESC);
