CREATE TABLE gift_codes (
  id INTEGER NOT NULL PRIMARY KEY,
  gift_code_b58 VARCHAR NOT NULL,
  entropy BLOB NOT NULL,
  txo_public_key BLOB NOT NULL,
  value UNSIGNED BIG INT NOT NULL,
  memo TEXT NOT NULL DEFAULT '',
  account_id_hex VARCHAR NOT NULL DEFAULT '',
  txo_id_hex VARCHAR NOT NULL,
  FOREIGN KEY (account_id_hex) REFERENCES accounts(account_id_hex),
  FOREIGN KEY (txo_id_hex) REFERENCES txos(txo_id_hex)
)
