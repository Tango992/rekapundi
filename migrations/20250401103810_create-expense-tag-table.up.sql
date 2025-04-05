-- Up
CREATE TABLE expense_tag (
    expense_id INTEGER NOT NULL REFERENCES expense(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tag(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (expense_id, tag_id)
);

CREATE INDEX ON expense_tag(expense_id);

CREATE INDEX ON expense_tag(tag_id);

CREATE TRIGGER update_expense_tag_updated_at
BEFORE UPDATE ON expense_tag
FOR EACH ROW
EXECUTE PROCEDURE update_updated_at_column();
