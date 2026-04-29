CREATE TABLE IF NOT EXISTS customers (
    id BINARY(16) NOT NULL PRIMARY KEY,
    email TEXT NOT NULL,
    street TEXT NOT NULL,
    zip_code TEXT NOT NULL
);
