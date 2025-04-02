-- Up
CREATE TABLE category (
    id SERIAL PRIMARY KEY,
    parent_category_id INTEGER NOT NULL REFERENCES parent_category(id),
    name TEXT NOT NULL,
    is_money_migration BOOLEAN NOT NULL
);

CREATE INDEX ON category(parent_category_id);

CREATE INDEX ON category(is_money_migration);
