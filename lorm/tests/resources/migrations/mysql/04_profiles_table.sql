CREATE TABLE IF NOT EXISTS profiles (
    id CHAR(36) NOT NULL PRIMARY KEY,
    user_id CHAR(36) NOT NULL,
    preferences JSON NOT NULL
);
