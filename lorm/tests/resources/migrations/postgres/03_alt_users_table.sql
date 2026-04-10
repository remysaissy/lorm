CREATE TABLE alt_users
(
    id         SERIAL PRIMARY KEY,
    e_mail     VARCHAR             NOT NULL UNIQUE,
    count      INTEGER,
    created_at TIMESTAMPTZ         NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ         NOT NULL
);
