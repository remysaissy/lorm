CREATE TABLE users
(
    id         TEXT PRIMARY KEY NOT NULL,
    email      VARCHAR          NOT NULL UNIQUE,
    count      INTEGER,
    created_at DATETIME         NOT NULL,
    updated_at DATETIME         NOT NULL
);