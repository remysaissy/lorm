CREATE TABLE IF NOT EXISTS posts (
    id        TEXT    PRIMARY KEY NOT NULL,
    title     TEXT    NOT NULL,
    published BOOLEAN NOT NULL DEFAULT FALSE,
    user_id   TEXT    NOT NULL REFERENCES users(id)
);
