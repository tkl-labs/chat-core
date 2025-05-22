-- Your SQL goes here
CREATE EXTENSION IF NOT EXISTS pgcrypto; -- For gen_random_uuid()

CREATE TABLE groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT, -- Description of group
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL,
    is_dm BOOLEAN NOT NULL DEFAULT false,
    CONSTRAINT fk_group_created_by FOREIGN KEY (created_by) REFERENCES users(id)
);

CREATE INDEX idx_groups_is_dm ON groups (is_dm);