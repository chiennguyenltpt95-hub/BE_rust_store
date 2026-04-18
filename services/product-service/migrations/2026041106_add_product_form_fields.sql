-- Add product fields required by storefront product form

ALTER TABLE products
ADD COLUMN IF NOT EXISTS supplier_id UUID,
ADD COLUMN IF NOT EXISTS supplier_url TEXT,
ADD COLUMN IF NOT EXISTS cost_price_cents BIGINT,
ADD COLUMN IF NOT EXISTS sale_price_cents BIGINT,
ADD COLUMN IF NOT EXISTS sizes TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
ADD COLUMN IF NOT EXISTS colors TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
ADD COLUMN IF NOT EXISTS weight_grams INTEGER,
ADD COLUMN IF NOT EXISTS tags TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[];

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'chk_products_cost_price_non_negative'
    ) THEN
        ALTER TABLE products
        ADD CONSTRAINT chk_products_cost_price_non_negative
        CHECK (cost_price_cents IS NULL OR cost_price_cents >= 0);
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'chk_products_sale_price_non_negative'
    ) THEN
        ALTER TABLE products
        ADD CONSTRAINT chk_products_sale_price_non_negative
        CHECK (sale_price_cents IS NULL OR sale_price_cents >= 0);
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'chk_products_weight_grams_non_negative'
    ) THEN
        ALTER TABLE products
        ADD CONSTRAINT chk_products_weight_grams_non_negative
        CHECK (weight_grams IS NULL OR weight_grams >= 0);
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'chk_products_sale_price_lte_price'
    ) THEN
        ALTER TABLE products
        ADD CONSTRAINT chk_products_sale_price_lte_price
        CHECK (sale_price_cents IS NULL OR sale_price_cents <= price_cents);
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'chk_products_cost_price_lte_price'
    ) THEN
        ALTER TABLE products
        ADD CONSTRAINT chk_products_cost_price_lte_price
        CHECK (cost_price_cents IS NULL OR cost_price_cents <= price_cents);
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_products_supplier_id ON products(supplier_id);
