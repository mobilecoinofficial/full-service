ALTER TABLE view_only_txos ADD COLUMN subaddress_index INTEGER;
ALTER TABLE view_only_txos ADD COLUMN submitted_block_index INTEGER;
ALTER TABLE view_only_txos ADD COLUMN pending_tombstone_block_index INTEGER;
ALTER TABLE view_only_txos ADD COLUMN received_block_index INTEGER;
ALTER TABLE view_only_txos ADD COLUMN spent_block_index INTEGER;

ALTER TABLE view_only_accounts ADD COLUMN next_subaddress_index INTEGER NOT NULL DEFAULT 2;
ALTER TABLE view_only_accounts ADD COLUMN main_subaddress_index INTEGER NOT NULL DEFAULT 0;
ALTER TABLE view_only_accounts ADD COLUMN change_subaddress_index INTEGER NOT NULL DEFAULT 1;

CREATE TABLE view_only_subaddresses (
  id INTEGER NOT NULL PRIMARY KEY,
  public_address_b58 TEXT NOT NULL UNIQUE,
  subaddress_index INT NOT NULL,
  view_only_account_id_hex TEXT NOT NULL,
  comment TEXT NOT NULL DEFAULT '',
  public_spend_key BLOB NOT NULL,
  FOREIGN KEY (view_only_account_id_hex) REFERENCES view_only_accounts(account_id_hex)
);