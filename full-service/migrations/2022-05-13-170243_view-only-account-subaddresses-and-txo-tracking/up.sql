DROP TABLE view_only_txos;
DROP TABLE view_only_accounts;

CREATE TABLE view_only_accounts (
  id INTEGER NOT NULL PRIMARY KEY,
  account_id_hex TEXT NOT NULL UNIQUE,
  view_private_key BLOB NOT NULL,
  first_block_index INTEGER NOT NULL,
  next_block_index INTEGER NOT NULL,
  import_block_index INTEGER NOT NULL,
  name TEXT NOT NULL DEFAULT '',
  next_subaddress_index INTEGER NOT NULL DEFAULT 2,
  main_subaddress_index INTEGER NOT NULL DEFAULT 0,
  change_subaddress_index INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE view_only_txos (
  id INTEGER NOT NULL PRIMARY KEY,
  txo_id_hex TEXT NOT NULL UNIQUE,
  txo BLOB NOT NULL,
  value INT NOT NULL,
  view_only_account_id_hex TEXT NOT NULL,
  public_key BLOB NOT NULL,
  subaddress_index INTEGER,
  submitted_block_index INTEGER,
  pending_tombstone_block_index INTEGER,
  received_block_index INTEGER,
  spent_block_index INTEGER,
  FOREIGN KEY (view_only_account_id_hex) REFERENCES view_only_accounts(account_id_hex)
);

CREATE TABLE view_only_subaddresses (
  id INTEGER NOT NULL PRIMARY KEY,
  public_address_b58 TEXT NOT NULL UNIQUE,
  subaddress_index INT NOT NULL,
  view_only_account_id_hex TEXT NOT NULL,
  comment TEXT NOT NULL DEFAULT '',
  public_spend_key BLOB NOT NULL,
  FOREIGN KEY (view_only_account_id_hex) REFERENCES view_only_accounts(account_id_hex)
);

DROP TABLE view_only_transaction_logs;