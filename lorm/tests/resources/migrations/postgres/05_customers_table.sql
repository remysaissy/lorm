CREATE TABLE IF NOT EXISTS customers (
    id UUID NOT NULL PRIMARY KEY,
    email TEXT NOT NULL,
    street TEXT NOT NULL,
    zip_code TEXT NOT NULL
);
