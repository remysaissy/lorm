CREATE TABLE IF NOT EXISTS opt_customers (
    id BINARY(16) NOT NULL PRIMARY KEY,
    email TEXT NOT NULL,
    street TEXT,
    zip_code TEXT
);
