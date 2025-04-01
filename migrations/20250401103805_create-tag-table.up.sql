-- Up
CREATE TABLE tag (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    is_marked_important BOOLEAN NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX ON tag(name);

CREATE INDEX ON tag(is_marked_important);

CREATE TRIGGER update_tag_updated_at
    BEFORE UPDATE ON tag
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
