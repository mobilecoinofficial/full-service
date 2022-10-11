DROP TABLE view_only_txos;
DROP TABLE view_only_subaddresses;
DROP TABLE view_only_accounts;
DROP TABLE assigned_subaddresses;
DROP TABLE transaction_txo_types;
DROP TABLE transaction_logs;
DROP TABLE txos;
DROP TABLE accounts;

CREATE TABLE accounts (
  id VARCHAR NOT NULL PRIMARY KEY,
  account_key BLOB NOT NULL,
  entropy BLOB,
  key_derivation_version INTEGER NOT NULL,
  first_block_index UNSIGNED BIG INT NOT NULL,
  next_block_index UNSIGNED BIG INT NOT NULL,
  import_block_index UNSIGNED BIG INT NULL,
  name VARCHAR NOT NULL DEFAULT '',
  fog_enabled BOOLEAN NOT NULL,
  view_only BOOLEAN NOT NULL
);

CREATE TABLE txos (
  id VARCHAR NOT NULL PRIMARY KEY,
  account_id VARCHAR,
  value UNSIGNED BIG INT NOT NULL,
  token_id UNSIGNED BIG INT NOT NULL,
  target_key BLOB NOT NULL,
  public_key BLOB NOT NULL,
  e_fog_hint BLOB NOT NULL,
  txo BLOB NOT NULL,
  subaddress_index UNSIGNED BIG INT,
  key_image BLOB,
  received_block_index UNSIGNED BIG INT,
  spent_block_index UNSIGNED BIG INT,
  shared_secret BLOB,
  FOREIGN KEY (account_id) REFERENCES accounts(id)
);

CREATE TABLE assigned_subaddresses (
  public_address_b58 VARCHAR NOT NULL PRIMARY KEY,
  account_id VARCHAR NOT NULL,
  subaddress_index UNSIGNED BIG INT NOT NULL,
  comment VARCHAR NOT NULL DEFAULT '',
  spend_public_key BLOB NOT NULL,
  FOREIGN KEY (account_id) REFERENCES accounts(id)
);

CREATE TABLE transaction_logs (
    id VARCHAR NOT NULL PRIMARY KEY,
    account_id VARCHAR NOT NULL,
    fee_value UNSIGNED BIG INT NOT NULL,
    fee_token_id UNSIGNED BIG INT NOT NULL,
    submitted_block_index UNSIGNED BIG INT,
    tombstone_block_index UNSIGNED BIG INT,
    finalized_block_index UNSIGNED BIG INT,
    comment TEXT NOT NULL DEFAULT '',
    tx BLOB NOT NULL,
    failed BOOLEAN NOT NULL,
    FOREIGN KEY (account_id) REFERENCES accounts(id)
);

CREATE TABLE transaction_input_txos (
  transaction_log_id VARCHAR NOT NULL,
  txo_id VARCHAR NOT NULL,
  PRIMARY KEY (transaction_log_id, txo_id),
  FOREIGN KEY (transaction_log_id) REFERENCES transaction_logs(id),
  FOREIGN KEY (txo_id) REFERENCES txos(id)
);

CREATE TABLE transaction_output_txos (
    transaction_log_id VARCHAR NOT NULL,
    txo_id VARCHAR NOT NULL,
    recipient_public_address_b58 VARCHAR NOT NULL,
    is_change BOOLEAN NOT NULL,
    PRIMARY KEY (transaction_log_id, txo_id),
    FOREIGN KEY (transaction_log_id) REFERENCES transaction_logs(id),
    FOREIGN KEY (txo_id) REFERENCES txos(id)
);
