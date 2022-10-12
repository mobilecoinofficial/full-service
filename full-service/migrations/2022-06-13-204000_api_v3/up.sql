DROP TABLE view_only_txos;
DROP TABLE view_only_subaddresses;
DROP TABLE view_only_accounts;

CREATE TABLE NEW_accounts (
  id VARCHAR NOT NULL PRIMARY KEY,
  account_key BLOB NOT NULL,
  entropy BLOB,
  key_derivation_version INTEGER NOT NULL,
  first_block_index UNSIGNED BIG INT NOT NULL,
  next_block_index UNSIGNED BIG INT NOT NULL,
  import_block_index UNSIGNED BIG INT NULL,
  name VARCHAR NOT NULL DEFAULT '',
  fog_enabled BOOLEAN NOT NULL,
  view_only BOOLEAN NOT NULL DEFAULT FALSE
);

INSERT INTO NEW_accounts SELECT account_id_hex, account_key, entropy, key_derivation_version, first_block_index, next_block_index, import_block_index, name, fog_enabled, FALSE FROM accounts;
DROP TABLE accounts;
ALTER TABLE NEW_accounts RENAME TO accounts;

CREATE TABLE NEW_assigned_subaddresses (
  public_address_b58 VARCHAR NOT NULL PRIMARY KEY,
  account_id VARCHAR NOT NULL,
  subaddress_index UNSIGNED BIG INT NOT NULL,
  comment VARCHAR NOT NULL DEFAULT '',
  spend_public_key BLOB NOT NULL,
  FOREIGN KEY (account_id) REFERENCES accounts(id)
);

INSERT INTO NEW_assigned_subaddresses SELECT assigned_subaddress_b58, account_id_hex, subaddress_index, comment, subaddress_spend_key FROM assigned_subaddresses;
DROP TABLE assigned_subaddresses;
ALTER TABLE NEW_assigned_subaddresses RENAME TO assigned_subaddresses;

CREATE TABLE transaction_input_txos (
  transaction_log_id VARCHAR NOT NULL,
  txo_id VARCHAR NOT NULL,
  PRIMARY KEY (transaction_log_id, txo_id),
  FOREIGN KEY (transaction_log_id) REFERENCES transaction_logs(id),
  FOREIGN KEY (txo_id) REFERENCES txos(id)
);

INSERT INTO transaction_input_txos SELECT transaction_id_hex, txo_id_hex FROM transaction_txo_types WHERE transaction_txo_type = 'txo_used_as_input';

CREATE TABLE transaction_output_txos (
    transaction_log_id VARCHAR NOT NULL,
    txo_id VARCHAR NOT NULL,
    recipient_public_address_b58 VARCHAR NOT NULL,
    is_change BOOLEAN NOT NULL,
    PRIMARY KEY (transaction_log_id, txo_id),
    FOREIGN KEY (transaction_log_id) REFERENCES transaction_logs(id),
    FOREIGN KEY (txo_id) REFERENCES txos(id)
);

INSERT INTO transaction_output_txos SELECT ttt.transaction_id_hex, ttt.txo_id_hex, txos.recipient_public_address_b58, FALSE 
FROM transaction_txo_types ttt
INNER JOIN transaction_logs tl ON ttt.transaction_id_hex = tl.transaction_id_hex
INNER JOIN txos ON ttt.txo_id_hex = txos.txo_id_hex
WHERE ttt.transaction_txo_type = 'txo_used_as_output' AND tl.direction = 'tx_direction_sent';

INSERT INTO transaction_output_txos SELECT ttt.transaction_id_hex, ttt.txo_id_hex, asub.public_address_b58, TRUE 
FROM transaction_txo_types ttt
INNER JOIN transaction_logs tl ON ttt.transaction_id_hex = tl.transaction_id_hex
INNER JOIN txos ON ttt.txo_id_hex = txos.txo_id_hex
INNER JOIN assigned_subaddresses asub ON asub.subaddress_index = txos.subaddress_index AND asub.account_id = txos.received_account_id_hex
WHERE ttt.transaction_txo_type = 'txo_used_as_change' AND tl.direction = 'tx_direction_sent';

DROP TABLE transaction_txo_types;

CREATE TABLE NEW_txos (
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

INSERT INTO NEW_txos SELECT txo_id_hex, received_account_id_hex, value, token_id, target_key, public_key, e_fog_hint, txo, subaddress_index, key_image, received_block_index, spent_block_index, confirmation FROM txos;
DROP TABLE txos;
ALTER TABLE NEW_txos RENAME TO txos;

CREATE TABLE NEW_transaction_logs (
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

INSERT INTO NEW_transaction_logs SELECT transaction_id_hex, account_id_hex, fee, 0, submitted_block_index, NULL, finalized_block_index, comment, tx, FALSE FROM transaction_logs WHERE direction = 'tx_direction_sent';
DROP TABLE transaction_logs;
ALTER TABLE NEW_transaction_logs RENAME TO transaction_logs;