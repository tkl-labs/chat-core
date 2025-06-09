CREATE EXTENSION IF NOT EXISTS pgcrypto; -- For gen_random_uuid()

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE CHECK (email ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$'),
    phone_number TEXT NOT NULL UNIQUE CHECK (phone_number ~ '^\+?[0-9]{7,15}$'),
    two_factor_auth BOOLEAN NOT NULL DEFAULT false,
    password_hash TEXT NOT NULL,
    profile_pic TEXT, -- Link to pfp img
    bio TEXT, -- Short text about the user
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

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

CREATE TABLE friend_request (
    requester UUID NOT NULL,
    receiver UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (requester, receiver),
    CONSTRAINT fk_friend_request_requester FOREIGN KEY (requester) REFERENCES users(id),
    CONSTRAINT fk_friend_request_receiver FOREIGN KEY (receiver) REFERENCES users(id)
);

CREATE TABLE friend (
    user1 UUID NOT NULL,
    user2 UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user1, user2),
    CONSTRAINT fk_friend_user1 FOREIGN KEY (user1) REFERENCES users(id),
    CONSTRAINT fk_friend_user2 FOREIGN KEY (user2) REFERENCES users(id)
);

CREATE INDEX idx_friend_user1 ON friend (user1);
CREATE INDEX idx_friend_user2 ON friend (user2);