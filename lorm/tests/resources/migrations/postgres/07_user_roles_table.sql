CREATE TABLE IF NOT EXISTS user_roles (
    user_id TEXT NOT NULL,
    role_id TEXT NOT NULL,
    assigned_at TEXT NOT NULL DEFAULT '',
    PRIMARY KEY (user_id, role_id)
);
