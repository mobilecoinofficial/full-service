DROP INDEX idx_transaction_logs__finalized_block_index;
ALTER TABLE transaction_logs RENAME COLUMN submitted_block_index TO submitted_block_count;
ALTER TABLE transaction_logs RENAME COLUMN finalized_block_index TO finalized_block_count;

-- ALTER TABLE accounts ALTER COLUMN entropy BLOB NOT NULL;
PRAGMA foreign_keys=OFF;
CREATE TABLE old_accounts (
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
INSERT INTO old_accounts SELECT * FROM accounts;
DROP TABLE accounts;
ALTER TABLE old_accounts RENAME TO accounts;
PRAGMA foreign_key_check;
PRAGMA foreign_keys=ON;
