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
  txo BLOB NOT NULL,
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

create TABLE assigned_subaddresses (
  assigned_subaddress_b58 VARCHAR NOT NULL PRIMARY KEY,
  account_id_hex VARCHAR NOT NULL,
  address_book_entry UNSIGNED BIG INT, -- FIXME add foreign key from address book table
  public_address BLOB NOT NULL,
  subaddress_index UNSIGNED BIG INT NOT NULL,
  comment VARCHAR NOT NULL DEFAULT '',
  expected_value UNSIGNED BIG INT,
  subaddress_spend_key BLOB NOT NULL,
  FOREIGN KEY (account_id_hex) REFERENCES accounts(account_id_hex)
);

create TABLE transaction_logs (
    transaction_id_hex VARCHAR NOT NULL PRIMARY KEY UNIQUE,
    account_id_hex VARCHAR NOT NULL,
    recipient_public_address_b58 VARCHAR NOT NULL DEFAULT '', -- FIXME add foreign key from recipient public addresses table
    assigned_subaddress_b58 VARCHAR NOT NULL DEFAULT '',
    value UNSIGNED BIG INT NOT NULL,
    fee UNSIGNED BIG INT,
    status VARCHAR(8) NOT NULL,
    sent_time VARCHAR NOT NULL DEFAULT '',
    block_height UNSIGNED BIG INT NOT NULL,
    comment TEXT NOT NULL DEFAULT '',
    direction VARCHAR(8) NOT NULL,
    FOREIGN KEY (account_id_hex) REFERENCES accounts(account_id_hex),
    FOREIGN KEY (assigned_subaddress_b58) REFERENCES assigned_subaddresses(assigned_subaddress_b58)
);

create TABLE transaction_txo_types (
    transaction_id_hex VARCHAR NOT NULL,
    txo_id_hex VARCHAR NOT NULL,
    transaction_txo_type VARCHAR(6) NOT NULL,
    PRIMARY KEY (transaction_id_hex, txo_id_hex),
    FOREIGN KEY (transaction_id_hex) REFERENCES transaction_logs(transaction_id_hex),
    FOREIGN KEY (txo_id_hex) REFERENCES txos(txo_id_hex)
)
