-- Your SQL goes here

CREATE TABLE friendships (
  user_id UUID NOT NULL,
  friend_id UUID NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  friendship_status TEXT CHECK (friendship_status IN ('pending', 'accepted', 'blocked')) NOT NULL,
  PRIMARY KEY (user_id, friend_id),
  CONSTRAINT fk_friend_user FOREIGN KEY (user_id) REFERENCES users(id),
  CONSTRAINT fk_friend_friend FOREIGN KEY (friend_id) REFERENCES users(id)
);

CREATE INDEX idx_friendship_user_id ON friendships (user_id);
CREATE INDEX idx_friendship_friend_id ON friendships (friend_id);