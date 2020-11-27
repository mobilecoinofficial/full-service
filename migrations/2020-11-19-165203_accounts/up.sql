CREATE TABLE accounts (
  account_id_hex VARCHAR NOT NULL PRIMARY KEY,
  encrypted_account_key BLOB NOT NULL,
  main_subaddress_index UNSIGNED BIG INT NOT NULL,
  change_subaddress_index UNSIGNED BIG INT NOT NULL,
  next_subaddress_index UNSIGNED BIG INT NOT NULL,
  first_block UNSIGNED BIG INT NOT NULL,
  next_block UNSIGNED BIG INT NOT NULL,
  name VARCHAR NOT NULL DEFAULT '',
  UNIQUE (account_id_hex)
);

create TABLE txos (
  txo_id_hex VARCHAR NOT NULL PRIMARY KEY UNIQUE,
  value UNSIGNED BIG INT NOT NULL,
  target_key BLOB NOT NULL,
  public_key BLOB NOT NULL,
  e_fog_hint BLOB NOT NULL,
  subaddress_index UNSIGNED BIG INT NOT NULL,
  key_image BLOB,
  received_block_height UNSIGNED BIG INT,
  spent_tombstone_block_height UNSIGNED BIG INT,
  spent_block_height UNSIGNED BIG INT,
  proof BLOB
);

create TABLE account_txo_statuses (
  account_id_hex VARCHAR NOT NULL,
  txo_id_hex VARCHAR NOT NULL,
  txo_status VARCHAR(8) NOT NULL,
  txo_type VARCHAR(7) NOT NULL,
  PRIMARY KEY (account_id_hex, txo_id_hex),
  FOREIGN KEY (account_id_hex) REFERENCES accounts(account_id_hex),
  FOREIGN KEY (txo_id_hex) REFERENCES txos(txo_id_hex)
);
