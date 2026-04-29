CREATE TABLE IF NOT EXISTS user_roles_named (
    user_id BLOB NOT NULL,
    role_name TEXT NOT NULL,
    PRIMARY KEY (user_id, role_name)
);
