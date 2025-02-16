CREATE TABLE posts
(
    id      TEXT PRIMARY KEY NOT NULL,
    content TEXT             NOT NULL UNIQUE,
    user_id TEXT             NOT NULL,
    FOREIGN KEY(user_id) REFERENCES users(id)
);