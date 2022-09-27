DROP INDEX idx_transaction_logs__finalized_block_index;

-- ALTER TABLE accounts RENAME COLUMN first_block_index TO first_block;
-- ALTER TABLE accounts RENAME COLUMN next_block_index TO next_block;
-- ALTER TABLE accounts RENAME COLUMN import_block_index TO import_block;
CREATE TABLE NEW_accounts (
  id INTEGER NOT NULL PRIMARY KEY,
  account_id_hex VARCHAR NOT NULL UNIQUE,
  account_key BLOB NOT NULL,
  entropy BLOB NOT NULL,
  main_subaddress_index UNSIGNED BIG INT NOT NULL,
  change_subaddress_index UNSIGNED BIG INT NOT NULL,
  next_subaddress_index UNSIGNED BIG INT NOT NULL,
  first_block UNSIGNED BIG INT NOT NULL,
  next_block UNSIGNED BIG INT NOT NULL,
  import_block UNSIGNED BIG INT,
  name VARCHAR NOT NULL DEFAULT ''
);
INSERT INTO NEW_accounts SELECT * FROM accounts;
DROP TABLE accounts;
ALTER TABLE NEW_accounts RENAME TO accounts;


-- ALTER TABLE txos RENAME COLUMN received_block_index TO received_block_count;
-- ALTER TABLE txos RENAME COLUMN pending_tombstone_block_index TO pending_tombstone_block_count;
-- ALTER TABLE txos RENAME COLUMN spent_block_index TO pending_tombstone_block_count;
CREATE TABLE NEW_txos (
  id INTEGER NOT NULL PRIMARY KEY,
  txo_id_hex VARCHAR NOT NULL UNIQUE,
  value UNSIGNED BIG INT NOT NULL,
  target_key BLOB NOT NULL,
  public_key BLOB NOT NULL,
  e_fog_hint BLOB NOT NULL,
  txo BLOB NOT NULL,
  subaddress_index UNSIGNED BIG INT,
  key_image BLOB,
  received_block_count UNSIGNED BIG INT,
  pending_tombstone_block_count UNSIGNED BIG INT,
  spent_block_count UNSIGNED BIG INT,
  proof BLOB
);
INSERT INTO NEW_txos SELECT * FROM txos;
DROP TABLE txos;
ALTER TABLE NEW_txos RENAME TO txos;

-- ALTER TABLE transaction_logs RENAME COLUMN submitted_block_index TO submitted_block_count;
-- ALTER TABLE transaction_logs RENAME COLUMN finalized_block_index TO finalized_block_count;
CREATE TABLE NEW_transaction_logs (
    id INTEGER NOT NULL PRIMARY KEY,
    transaction_id_hex VARCHAR NOT NULL UNIQUE,
    account_id_hex VARCHAR NOT NULL,
    recipient_public_address_b58 VARCHAR NOT NULL DEFAULT '',
    assigned_subaddress_b58 VARCHAR NOT NULL DEFAULT '',
    value UNSIGNED BIG INT NOT NULL,
    fee UNSIGNED BIG INT,
    status VARCHAR(8) NOT NULL,
    sent_time UNSIGNED BIG INT,
    submitted_block_count UNSIGNED BIG INT,
    finalized_block_count UNSIGNED BIG INT,
    comment TEXT NOT NULL DEFAULT '',
    direction VARCHAR(8) NOT NULL,
    tx BLOB,
    FOREIGN KEY (account_id_hex) REFERENCES accounts(account_id_hex),
    FOREIGN KEY (assigned_subaddress_b58) REFERENCES assigned_subaddresses(assigned_subaddress_b58)
);
INSERT INTO NEW_transaction_logs SELECT * FROM transaction_logs;
DROP TABLE transaction_logs;
ALTER TABLE NEW_transaction_logs RENAME TO transaction_logs;
