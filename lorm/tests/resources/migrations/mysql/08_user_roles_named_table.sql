CREATE TABLE IF NOT EXISTS user_roles_named (
    user_id BINARY(16) NOT NULL,
    role_name VARCHAR(255) NOT NULL,
    PRIMARY KEY (user_id, role_name)
);
