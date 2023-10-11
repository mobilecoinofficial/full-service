ALTER TABLE txos ADD COLUMN memo_type INTEGER NULL;

CREATE TABLE authenticated_sender_memos (
  txo_id TEXT PRIMARY KEY NOT NULL,
  sender_address_hash TEXT NOT NULL,
  payment_request_id TEXT,
  payment_intent_id TEXT,
  FOREIGN KEY (txo_id) REFERENCES txos(id)
);

UPDATE accounts SET next_block_index = 0;