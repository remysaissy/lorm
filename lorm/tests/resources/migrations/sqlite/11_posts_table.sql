CREATE TABLE IF NOT EXISTS posts (
    id        TEXT    PRIMARY KEY NOT NULL,
    title     TEXT    NOT NULL,
    published INTEGER NOT NULL DEFAULT 0,
    user_id   TEXT    NOT NULL REFERENCES users(id)
);
