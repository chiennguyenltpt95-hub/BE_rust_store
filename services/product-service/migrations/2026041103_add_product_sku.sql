-- Add SKU for product identity and better searching

ALTER TABLE products
ADD COLUMN IF NOT EXISTS sku VARCHAR(64);

CREATE UNIQUE INDEX IF NOT EXISTS uq_products_sku
ON products(sku)
WHERE sku IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_products_name_status
ON products(name, status);
