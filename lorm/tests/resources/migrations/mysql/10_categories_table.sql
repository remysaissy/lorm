CREATE TABLE IF NOT EXISTS categories (
    id        BINARY(16) PRIMARY KEY NOT NULL,
    name      TEXT       NOT NULL,
    parent_id BINARY(16)
);
