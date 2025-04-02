-- Up
CREATE TABLE income (
    id SERIAL PRIMARY KEY,
    amount INTEGER NOT NULL,
    wallet_id INTEGER NOT NULL REFERENCES wallet(id),
    date DATE NOT NULL,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX ON income(wallet_id);

CREATE INDEX ON income(date);

CREATE TRIGGER update_income_updated_at
BEFORE UPDATE ON income
FOR EACH ROW
EXECUTE PROCEDURE update_updated_at_column();
