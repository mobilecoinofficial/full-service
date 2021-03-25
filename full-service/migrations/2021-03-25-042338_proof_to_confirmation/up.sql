-- ALTER TABLE txos RENAME COLUMN proof TO confirmation;
PRAGMA foreign_keys=OFF;
CREATE TABLE NEW_txos (
    id INTEGER NOT NULL PRIMARY KEY,
    txo_id_hex VARCHAR NOT NULL UNIQUE,
    value UNSIGNED BIG INT NOT NULL,
    target_key BLOB NOT NULL,
    public_key BLOB NOT NULL,
    e_fog_hint BLOB NOT NULL,
    txo BLOB NOT NULL,
    subaddress_index UNSIGNED BIG INT,
    key_image BLOB,
    received_block_index UNSIGNED BIG INT,
    pending_tombstone_block_index UNSIGNED BIG INT,
    spent_block_index UNSIGNED BIG INT,
    confirmation BLOB
);
INSERT INTO NEW_txos SELECT * FROM txos;
DROP TABLE txos;
ALTER TABLE NEW_txos RENAME TO txos;
PRAGMA foreign_key_check;
PRAGMA foreign_keys=ON;
