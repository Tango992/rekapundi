-- Up
CREATE TABLE wallet_transfer (
    id SERIAL PRIMARY KEY,
    amount INTEGER NOT NULL,
    source_wallet_id INTEGER NOT NULL REFERENCES wallet(id),
    target_wallet_id INTEGER NOT NULL REFERENCES wallet(id),
    date DATE NOT NULL,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT source_target_wallet_different CHECK (source_wallet_id <> target_wallet_id)
);

CREATE TRIGGER update_wallet_transfer_updated_at
BEFORE UPDATE ON wallet_transfer
FOR EACH ROW
EXECUTE PROCEDURE update_updated_at_column();
