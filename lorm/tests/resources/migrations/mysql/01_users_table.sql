CREATE TABLE users
(
    id         BINARY(16) PRIMARY KEY  NOT NULL,
    email      VARCHAR(255)            NOT NULL UNIQUE,
    count      INTEGER,
    created_at TIMESTAMP(6)            NOT NULL,
    updated_at TIMESTAMP(6)            NOT NULL
);
