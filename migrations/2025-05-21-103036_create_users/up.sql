-- Your SQL goes here
CREATE EXTENSION IF NOT EXISTS pgcrypto; -- For gen_random_uuid()

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE CHECK (email ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$'),
    phone_number TEXT NOT NULL CHECK (phone_number ~ '^\+?[0-9]{7,15}$'),
    two_factor_auth BOOLEAN NOT NULL DEFAULT false,
    password_hash TEXT NOT NULL,
    profile_pic TEXT, -- Link to pfp img
    bio TEXT, -- Short text about the user
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);