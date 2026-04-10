CREATE TABLE users
(
    id         UUID PRIMARY KEY NOT NULL,
    email      VARCHAR          NOT NULL UNIQUE,
    count      INTEGER,
    created_at TIMESTAMPTZ      NOT NULL,
    updated_at TIMESTAMPTZ      NOT NULL
);
