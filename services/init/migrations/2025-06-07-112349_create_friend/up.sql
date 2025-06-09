-- Your SQL goes here
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