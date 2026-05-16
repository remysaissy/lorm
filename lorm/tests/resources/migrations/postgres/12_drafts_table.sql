CREATE TABLE IF NOT EXISTS drafts (
    id      TEXT PRIMARY KEY NOT NULL,
    title   TEXT NOT NULL,
    user_id TEXT REFERENCES users(id)
);
