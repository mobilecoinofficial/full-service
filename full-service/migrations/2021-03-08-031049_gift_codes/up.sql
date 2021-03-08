CREATE TABLE gift_codes (
  id INTEGER NOT NULL PRIMARY KEY,
  gift_code_b58 VARCHAR NOT NULL,
  entropy BLOB NOT NULL,
  txo_public_key BLOB NOT NULL,
  value UNSIGNED BIG INT NOT NULL,
  memo TEXT NOT NULL DEFAULT '',
  account_id INTEGER NOT NULL,
  build_log_id INTEGER,
  consume_log_id INTEGER,
  FOREIGN KEY (account_id) REFERENCES accounts(id),
  FOREIGN KEY (build_log_id) REFERENCES transaction_logs(id),
  FOREIGN KEY (consume_log_id) REFERENCES transaction_logs(id)
)