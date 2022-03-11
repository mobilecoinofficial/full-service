-- Your SQL goes here
CREATE TABLE view_only_txos (
  id INTEGER NOT NULL PRIMARY KEY,
  txo_id_hex TEXT NOT NULL UNIQUE,
  txo BLOB NOT NULL,
  value INT NOT NULL,
  view_only_account_id_hex TEXT NOT NULL,
  spent BOOLEAN NOT NULL DEFAULT FALSE,
  FOREIGN KEY (view_only_account_id_hex) REFERENCES view_only_accounts(account_id_hex)
);