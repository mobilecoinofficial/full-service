CREATE TABLE gift_codes (
  id INTEGER NOT NULL PRIMARY KEY,
  gift_code_b58 VARCHAR NOT NULL,
  entropy BLOB NOT NULL,
  txo_public_key BLOB NOT NULL,
  value UNSIGNED BIG INT NOT NULL,
  memo TEXT NOT NULL DEFAULT '',
  account_id_hex VARCHAR NOT NULL DEFAULT '',
  build_log_id_hex VARCHAR NOT NULL DEFAULT '',
  claim_log_id_hex VARCHAR NOT NULL DEFAULT '',
  FOREIGN KEY (account_id_hex) REFERENCES accounts(account_id_hex),
  FOREIGN KEY (build_log_id_hex) REFERENCES transaction_logs(transaction_id_hex),
  FOREIGN KEY (claim_log_id_hex) REFERENCES transaction_logs(transaction_id_hex)
)