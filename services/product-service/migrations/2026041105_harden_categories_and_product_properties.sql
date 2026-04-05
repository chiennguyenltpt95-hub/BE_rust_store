-- Forward-only migration to extend categories and products metadata/properties

ALTER TABLE categories
ALTER COLUMN slug TYPE VARCHAR(140);

ALTER TABLE categories
ADD COLUMN IF NOT EXISTS description TEXT,
ADD COLUMN IF NOT EXISTS image_url TEXT,
ADD COLUMN IF NOT EXISTS parent_id UUID,
ADD COLUMN IF NOT EXISTS display_order INTEGER NOT NULL DEFAULT 0,
ADD COLUMN IF NOT EXISTS is_active BOOLEAN NOT NULL DEFAULT TRUE,
ADD COLUMN IF NOT EXISTS target_gender VARCHAR(20) NOT NULL DEFAULT 'female';

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'chk_categories_display_order_non_negative'
    ) THEN
        ALTER TABLE categories
        ADD CONSTRAINT chk_categories_display_order_non_negative
        CHECK (display_order >= 0);
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'chk_categories_target_gender'
    ) THEN
        ALTER TABLE categories
        ADD CONSTRAINT chk_categories_target_gender
        CHECK (target_gender IN ('female', 'unisex'));
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'fk_categories_parent_id'
    ) THEN
        ALTER TABLE categories
        ADD CONSTRAINT fk_categories_parent_id
        FOREIGN KEY (parent_id)
        REFERENCES categories(id)
        ON DELETE SET NULL;
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_categories_parent_id ON categories(parent_id);
CREATE INDEX IF NOT EXISTS idx_categories_is_active ON categories(is_active);
CREATE INDEX IF NOT EXISTS idx_categories_display_order ON categories(display_order);
CREATE INDEX IF NOT EXISTS idx_categories_target_gender ON categories(target_gender);

CREATE OR REPLACE FUNCTION set_categories_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_set_categories_updated_at ON categories;

CREATE TRIGGER trg_set_categories_updated_at
BEFORE UPDATE ON categories
FOR EACH ROW
EXECUTE FUNCTION set_categories_updated_at();

ALTER TABLE products
ADD COLUMN IF NOT EXISTS product_type VARCHAR(60),
ADD COLUMN IF NOT EXISTS attributes JSONB NOT NULL DEFAULT '{}'::jsonb,
ADD COLUMN IF NOT EXISTS image_urls TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[];

CREATE INDEX IF NOT EXISTS idx_products_product_type ON products(product_type);
CREATE INDEX IF NOT EXISTS idx_products_attributes_gin ON products USING GIN(attributes);
