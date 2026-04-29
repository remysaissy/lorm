CREATE TABLE IF NOT EXISTS profiles (
    id TEXT NOT NULL PRIMARY KEY,
    user_id TEXT NOT NULL,
    preferences TEXT NOT NULL
);
