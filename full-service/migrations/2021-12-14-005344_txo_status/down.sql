CREATE TABLE account_txo_statuses (
      account_id_hex TEXT NOT NULL,
      txo_id_hex TEXT NOT NULL,
      txo_status TEXT NOT NULL,
      txo_type TEXT NOT NULL,
      PRIMARY KEY (account_id_hex, txo_id_hex),
      FOREIGN KEY (account_id_hex) REFERENCES accounts(account_id_hex),
      FOREIGN KEY (txo_id_hex) REFERENCES txos(txo_id_hex)
);

-- todo: reverse data migration

ALTER TABLE txos REMOVE COLUMN minted_account_id_hex;
ALTER TABLE txos REMOVE COLUMN received_account_id_hex;

