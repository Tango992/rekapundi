-- Up
CREATE TABLE tag (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    is_important BOOLEAN NOT NULL
);

CREATE INDEX ON tag(is_important);
