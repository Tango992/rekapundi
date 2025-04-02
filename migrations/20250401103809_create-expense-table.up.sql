-- Up
CREATE TABLE expense (
    id SERIAL PRIMARY KEY,
    category_id INTEGER NOT NULL REFERENCES category(id),
    wallet_id INTEGER NOT NULL REFERENCES wallet(id),
    amount INTEGER NOT NULL,
    date DATE NOT NULL,
    description TEXT,
    CREATED_AT TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UPDATED_AT TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX ON expense(category_id);

CREATE INDEX ON expense(wallet_id);

CREATE INDEX ON expense(date);

CREATE TRIGGER update_expense_updated_at
BEFORE UPDATE ON expense
FOR EACH ROW
EXECUTE PROCEDURE update_updated_at_column();
