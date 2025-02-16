CREATE TABLE alt_users
(
    id         INTEGER PRIMARY KEY NOT NULL,
    email      VARCHAR          NOT NULL UNIQUE,
    count      INTEGER,
    created_at DATETIME         NOT NULL DEFAULT(DATETIME('now')),
    updated_at DATETIME         NOT NULL
);
