db = db.getSiblingDB("tkl-chat");
db.createCollection("messages");

db.messages.createIndex(
  { group_id: 1, timestamp: 1, message_id: 1 },
  { unique: true }
);

// when adding a message, use:
// group_id UUID,
// message_id UUID,
// sender_id UUID,
// timestamp TIMESTAMP,
// content TEXT,
