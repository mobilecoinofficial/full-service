ALTER TABLE view_only_txos 
ADD COLUMN subaddress_index INTEGER
ADD COLUMN submitted_block_index INTEGER
ADD COLUMN pending_tombstone_block_index INTEGER
ADD COLUMN received_block_index INTEGER
ADD COLUMN spent_block_index INTEGER;

CREATE TABLE view_only_subaddresses (
  id INTEGER NOT NULL PRIMARY KEY,
  public_address_b58 TEXT NOT NULL UNIQUE,
  subaddress_index INT NOT NULL,
  view_only_account_id_hex TEXT NOT NULL,
  comment TEXT NOT NULL DEFAULT '',
  public_spend_key BLOB NOT NULL,
  FOREIGN KEY (view_only_account_id_hex) REFERENCES view_only_accounts(account_id_hex)
);