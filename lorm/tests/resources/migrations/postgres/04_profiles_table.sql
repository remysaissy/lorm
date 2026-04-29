CREATE TABLE IF NOT EXISTS profiles (
    id UUID NOT NULL PRIMARY KEY,
    user_id UUID NOT NULL,
    preferences JSONB NOT NULL
);
