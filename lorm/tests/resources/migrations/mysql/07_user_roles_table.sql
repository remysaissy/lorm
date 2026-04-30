CREATE TABLE IF NOT EXISTS user_roles (
    user_id VARCHAR(255) NOT NULL,
    role_id VARCHAR(255) NOT NULL,
    assigned_at VARCHAR(255) NOT NULL DEFAULT '',
    PRIMARY KEY (user_id, role_id)
);
