CREATE TABLE IF NOT EXISTS categories (
    id         UUID PRIMARY KEY NOT NULL,
    name       TEXT             NOT NULL,
    parent_id  UUID REFERENCES categories(id)
);
