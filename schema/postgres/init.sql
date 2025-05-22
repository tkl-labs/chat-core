CREATE EXTENSION IF NOT EXISTS pgcrypto; -- For gen_random_uuid()

CREATE TABLE "user" (
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

CREATE TABLE "group" (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT, -- Description of group
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL,
    is_dm BOOLEAN NOT NULL DEFAULT false,
    CONSTRAINT fk_group_created_by FOREIGN KEY (created_by) REFERENCES "user" (id)
);

CREATE TABLE "group_member" (
    user_id UUID NOT NULL,
    group_id UUID NOT NULL,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    role TEXT NOT NULL DEFAULT 'member', -- Admin / staff / member, etc
    PRIMARY KEY (user_id, group_id),
    CONSTRAINT fk_member_user FOREIGN KEY (user_id) REFERENCES "user" (id),
    CONSTRAINT fk_member_group FOREIGN KEY (group_id) REFERENCES "group" (id)
);

CREATE INDEX idx_group_member_group_id ON group_member (group_id);