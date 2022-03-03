CREATE TABLE view_only_accounts (
  id INTEGER NOT NULL PRIMARY KEY,
  view_private_key BLOB NOT NULL,
  first_block_index UNSIGNED BIG INT NOT NULL,
  next_block_index UNSIGNED BIG INT NOT NULL,
  import_block_index UNSIGNED BIG INT NOT NULL,
  name VARCHAR NOT NULL DEFAULT ''
);

