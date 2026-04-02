CREATE OR REPLACE FUNCTION set_checkouts_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_set_checkouts_updated_at ON checkouts;

CREATE TRIGGER trg_set_checkouts_updated_at
BEFORE UPDATE ON checkouts
FOR EACH ROW
EXECUTE FUNCTION set_checkouts_updated_at();
