CREATE TABLE IF NOT EXISTS posts (
    id        BINARY(16)  PRIMARY KEY NOT NULL,
    title     TEXT        NOT NULL,
    published TINYINT(1)  NOT NULL DEFAULT 0,
    user_id   BINARY(16)  NOT NULL
);
