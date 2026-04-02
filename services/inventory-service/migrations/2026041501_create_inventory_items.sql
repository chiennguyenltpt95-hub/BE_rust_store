CREATE TABLE IF NOT EXISTS inventory_items (
    product_id UUID PRIMARY KEY,
    available_qty INTEGER NOT NULL CHECK (available_qty >= 0),
    reserved_qty INTEGER NOT NULL DEFAULT 0 CHECK (reserved_qty >= 0),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_inventory_items_updated_at ON inventory_items(updated_at DESC);
