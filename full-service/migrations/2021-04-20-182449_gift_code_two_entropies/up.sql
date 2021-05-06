PRAGMA foreign_keys=OFF;

CREATE TABLE NEW_gift_codes (
  id INTEGER NOT NULL PRIMARY KEY,
  gift_code_b58 VARCHAR NOT NULL,
  root_entropy BLOB,
  bip39_entropy BLOB,
  txo_public_key BLOB NOT NULL,
  value UNSIGNED BIG INT NOT NULL,
  memo TEXT NOT NULL DEFAULT '',
  account_id_hex VARCHAR NOT NULL DEFAULT '',
  txo_id_hex VARCHAR NOT NULL,
  FOREIGN KEY (account_id_hex) REFERENCES accounts(account_id_hex),
  FOREIGN KEY (txo_id_hex) REFERENCES txos(txo_id_hex)
);

INSERT INTO NEW_gift_codes SELECT
  id,
  gift_code_b58,
  entropy,
  NULL,
  txo_public_key,
  value,
  memo,
  account_id_hex,
  txo_id_hex
FROM gift_codes;

DROP TABLE gift_codes;
ALTER TABLE NEW_gift_codes RENAME TO gift_codes;

PRAGMA foreign_key_check;
PRAGMA foreign_keys=ON;
