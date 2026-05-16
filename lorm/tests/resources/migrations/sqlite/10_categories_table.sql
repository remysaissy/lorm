CREATE TABLE IF NOT EXISTS categories (
    id         TEXT PRIMARY KEY NOT NULL,
    name       TEXT             NOT NULL,
    parent_id  TEXT REFERENCES categories(id)
);
