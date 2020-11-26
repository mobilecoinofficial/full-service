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
