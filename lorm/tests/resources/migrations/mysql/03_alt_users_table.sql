CREATE TABLE alt_users
(
    id         INTEGER PRIMARY KEY AUTO_INCREMENT,
    e_mail     VARCHAR(255)        NOT NULL UNIQUE,
    count      INTEGER,
    created_at TIMESTAMP(6)        NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    updated_at TIMESTAMP(6)        NOT NULL
);
