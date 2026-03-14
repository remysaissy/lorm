CREATE TABLE pairings (
    user1 INTEGER NOT NULL,
    user2 INTEGER NOT NULL,
    score_value INTEGER NOT NULL,
    PRIMARY KEY (user1, user2)
);