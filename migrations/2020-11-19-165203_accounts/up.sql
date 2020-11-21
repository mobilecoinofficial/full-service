CREATE TABLE accounts (
  account_id_hex VARCHAR NOT NULL PRIMARY KEY,
  encrypted_account_key BLOB NOT NULL,
  main_subaddress_index VARCHAR NOT NULL DEFAULT '0',
  change_subaddress_index VARCHAR NOT NULL DEFAULT '1',
  next_subaddress_index VARCHAR NOT NULL DEFAULT '2',
  first_block VARCHAR NOT NULL DEFAULT '0',
  next_block VARCHAR NOT NULL DEFAULT '1',
  name TEXT
);
