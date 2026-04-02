CREATE TABLE IF NOT EXISTS shipments (
    id UUID PRIMARY KEY,
    order_id UUID NOT NULL,
    carrier VARCHAR(64),
    tracking_code VARCHAR(128),
    status VARCHAR(20) NOT NULL CHECK (status IN ('pending', 'packed', 'shipped', 'delivered', 'failed')),
    shipping_address TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_shipments_order_id ON shipments(order_id);
CREATE INDEX IF NOT EXISTS idx_shipments_status_updated ON shipments(status, updated_at DESC);
