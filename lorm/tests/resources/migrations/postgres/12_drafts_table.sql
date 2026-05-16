CREATE TABLE IF NOT EXISTS drafts (
    id      UUID PRIMARY KEY NOT NULL,
    title   TEXT NOT NULL,
    user_id UUID REFERENCES users(id)
);
