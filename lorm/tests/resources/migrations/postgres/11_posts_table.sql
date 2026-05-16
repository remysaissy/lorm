CREATE TABLE IF NOT EXISTS posts (
    id        UUID    PRIMARY KEY NOT NULL,
    title     TEXT    NOT NULL,
    published BOOLEAN NOT NULL DEFAULT FALSE,
    user_id   UUID    NOT NULL REFERENCES users(id)
);
