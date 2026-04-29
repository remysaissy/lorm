CREATE TABLE IF NOT EXISTS user_roles (
    user_id BLOB NOT NULL,
    role_id BLOB NOT NULL,
    PRIMARY KEY (user_id, role_id)
);
