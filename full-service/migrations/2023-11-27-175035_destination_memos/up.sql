CREATE TABLE destination_memos (
  txo_id TEXT PRIMARY KEY NOT NULL,
  recipient_address_hash TEXT NOT NULL,
  num_recipients INT NOT NULL,
  fee UNSIGNED BIG INT NOT NULL,
  total_outlay UNSIGNED BIG INT NOT NULL,
  payment_request_id UNSIGNED BIG INT,
  payment_intent_id UNSIGNED BIG INT,
  FOREIGN KEY (txo_id) REFERENCES txos(id)
);

UPDATE accounts SET next_block_index = 0;
UPDATE accounts SET resyncing = TRUE;