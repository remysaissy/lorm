CREATE TABLE IF NOT EXISTS drafts (
    id      BINARY(16) PRIMARY KEY NOT NULL,
    title   TEXT       NOT NULL,
    user_id BINARY(16)
);
