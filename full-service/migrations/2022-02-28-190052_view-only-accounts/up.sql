CREATE TABLE view_only_accounts (
  id INTEGER NOT NULL PRIMARY KEY,
  account_id_hex TEXT NOT NULL UNIQUE,
  view_private_key BLOB NOT NULL,
  first_block_index INTEGER NOT NULL,
  next_block_index INTEGER NOT NULL,
  import_block_index INTEGER NOT NULL,
  name TEXT NOT NULL DEFAULT ''
);

