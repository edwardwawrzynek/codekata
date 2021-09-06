CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email TEXT,
    name TEXT NOT NULL,
    is_admin BOOLEAN NOT NULL,
    password_hash TEXT,
    api_key_hash TEXT NOT NULL
)