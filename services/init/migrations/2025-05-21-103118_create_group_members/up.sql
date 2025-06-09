-- Your SQL goes here
CREATE EXTENSION IF NOT EXISTS pgcrypto; -- For gen_random_uuid()

CREATE TABLE group_members (
    user_id UUID NOT NULL,
    group_id UUID NOT NULL,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    role TEXT NOT NULL DEFAULT 'member', -- Admin / staff / member, etc
    PRIMARY KEY (user_id, group_id),
    CONSTRAINT fk_member_user FOREIGN KEY (user_id) REFERENCES users(id),
    CONSTRAINT fk_member_group FOREIGN KEY (group_id) REFERENCES groups(id)
);

CREATE INDEX idx_group_member_group_id ON group_members (group_id);