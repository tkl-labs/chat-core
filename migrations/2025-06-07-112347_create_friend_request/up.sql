-- Your SQL goes here
CREATE TABLE friend_request (
    requester UUID NOT NULL,
    receiver UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (requester, receiver),
    CONSTRAINT fk_friend_request_requester FOREIGN KEY (requester) REFERENCES users(id),
    CONSTRAINT fk_friend_request_receiver FOREIGN KEY (receiver) REFERENCES users(id)
);