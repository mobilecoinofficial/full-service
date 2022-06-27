CREATE TABLE accounts (
  id INTEGER NOT NULL PRIMARY KEY,
  account_id_hex VARCHAR NOT NULL UNIQUE,
  account_key BLOB NOT NULL,
  entropy BLOB,
  key_derivation_version INTEGER NOT NULL,
  main_subaddress_index UNSIGNED BIG INT NOT NULL,
  change_subaddress_index UNSIGNED BIG INT NOT NULL,
  next_subaddress_index UNSIGNED BIG INT NOT NULL,
  first_block_index UNSIGNED BIG INT NOT NULL,
  next_block_index UNSIGNED BIG INT NOT NULL,
  import_block_index UNSIGNED BIG INT NULL,
  name VARCHAR NOT NULL DEFAULT '',
  fog_enabled BOOLEAN NOT NULL,
  view_only BOOLEAN NOT NULL
);

CREATE UNIQUE INDEX idx_accounts__account_id_hex ON accounts (account_id_hex);

CREATE TABLE txos (
  id INTEGER NOT NULL PRIMARY KEY,
  txo_id_hex VARCHAR NOT NULL UNIQUE,
  value UNSIGNED BIG INT NOT NULL,
  token_id UNSIGNED BIG INT NOT NULL,
  target_key BLOB NOT NULL,
  public_key BLOB NOT NULL,
  e_fog_hint BLOB NOT NULL,
  txo BLOB NOT NULL,
  subaddress_index UNSIGNED BIG INT,
  key_image BLOB,
  received_block_index UNSIGNED BIG INT,
  pending_tombstone_block_index UNSIGNED BIG INT,
  spent_block_index UNSIGNED BIG INT,
  confirmation BLOB,
  recipient_public_address_b58 VARCHAR NOT NULL,
  minted_account_id_hex VARCHAR,
  received_account_id_hex VARCHAR,
  FOREIGN KEY (minted_account_id_hex) REFERENCES accounts(account_id_hex),
  FOREIGN KEY (received_account_id_hex) REFERENCES accounts(account_id_hex)
);

CREATE UNIQUE INDEX idx_txos__txo_id_hex ON txos (txo_id_hex);

CREATE TABLE assigned_subaddresses (
  id INTEGER NOT NULL PRIMARY KEY,
  assigned_subaddress_b58 VARCHAR NOT NULL UNIQUE,
  account_id_hex VARCHAR NOT NULL,
  address_book_entry UNSIGNED BIG INT, -- FIXME: WS-8 add foreign key to address book table, also address_book_entry_id
  public_address BLOB NOT NULL,
  subaddress_index UNSIGNED BIG INT NOT NULL,
  comment VARCHAR NOT NULL DEFAULT '',
  subaddress_spend_key BLOB NOT NULL,
  FOREIGN KEY (account_id_hex) REFERENCES accounts(account_id_hex)
);

CREATE UNIQUE INDEX idx_assigned_subaddresses__assigned_subaddress_b58 ON assigned_subaddresses (assigned_subaddress_b58);

CREATE TABLE transaction_logs (
    id INTEGER NOT NULL PRIMARY KEY,
    transaction_id_hex VARCHAR NOT NULL UNIQUE,
    account_id_hex VARCHAR NOT NULL,
    assigned_subaddress_b58 VARCHAR,
    value UNSIGNED BIG INT NOT NULL,
    fee UNSIGNED BIG INT,
    status VARCHAR(8) NOT NULL,
    sent_time UNSIGNED BIG INT,
    submitted_block_index UNSIGNED BIG INT,
    finalized_block_index UNSIGNED BIG INT,
    comment TEXT NOT NULL DEFAULT '',
    direction VARCHAR(8) NOT NULL,
    tx BLOB,
    FOREIGN KEY (account_id_hex) REFERENCES accounts(account_id_hex),
    FOREIGN KEY (assigned_subaddress_b58) REFERENCES assigned_subaddresses(assigned_subaddress_b58)
);

CREATE UNIQUE INDEX idx_transaction_logs__transaction_id_hex ON transaction_logs (transaction_id_hex);

CREATE TABLE transaction_txo_types (
    transaction_id_hex VARCHAR NOT NULL,
    txo_id_hex VARCHAR NOT NULL,
    transaction_txo_type VARCHAR(6) NOT NULL,
    PRIMARY KEY (transaction_id_hex, txo_id_hex),
    FOREIGN KEY (transaction_id_hex) REFERENCES transaction_logs(transaction_id_hex),
    FOREIGN KEY (txo_id_hex) REFERENCES txos(txo_id_hex)
);

CREATE TABLE gift_codes (
    id INTEGER NOT NULL PRIMARY KEY,
    gift_code_b58 VARCHAR NOT NULL UNIQUE,
    value BIG INT NOT NULL
);
