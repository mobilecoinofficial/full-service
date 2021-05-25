-- Add the recipient address column to txos.
-- ALTER TABLE txos ADD COLUMN recipient_public_address_b58 TEXT NOT NULL DEFAULT '';
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
    confirmation BLOB,
    recipient_public_address_b58 TEXT NOT NULL DEFAULT ''
);
INSERT INTO NEW_txos SELECT
    id,
    txo_id_hex,
    value,
    target_key,
    public_key,
    e_fog_hint,
    txo,
    subaddress_index,
    key_image,
    received_block_index,
    pending_tombstone_block_index,
    spent_block_index,
    confirmation,
    ''
FROM txos;
DROP TABLE txos;
ALTER TABLE NEW_txos RENAME TO txos;
PRAGMA foreign_key_check;
PRAGMA foreign_keys=ON;

-- Update the txos table with the relevant values from transaction_logs for recipient_public_address_b58.
UPDATE txos
SET recipient_public_address_b58 = q.recipient_public_address_b58
FROM (
    SELECT txos.txo_id_hex, tl.recipient_public_address_b58
    FROM transaction_txo_types AS ttt
        JOIN txos ON ttt.txo_id_hex = txos.txo_id_hex
        JOIN transaction_logs AS tl ON ttt.transaction_id_hex = tl.transaction_id_hex
    WHERE tl.recipient_public_address_b58 != '' AND ttt.transaction_txo_type = 'txo_used_as_output'
) AS q
WHERE txos.txo_id_hex = q.txo_id_hex;

-- Remove the recipient address column from transaction logs.
-- ALTER TABLE transaction_logs REMOVE COLUMN recipient_public_address_b58;
PRAGMA foreign_keys=OFF;
CREATE TABLE NEW_transaction_logs (
    id INTEGER NOT NULL PRIMARY KEY,
    transaction_id_hex VARCHAR NOT NULL UNIQUE,
    account_id_hex VARCHAR NOT NULL,
    assigned_subaddress_b58 VARCHAR NULL,
    value UNSIGNED BIG INT NOT NULL,
    fee UNSIGNED BIG INT,
    status VARCHAR(8) NOT NULL,
    sent_time UNSIGNED BIG INT,
    submitted_block_index UNSIGNED BIG INT,
    finalized_block_index UNSIGNED BIG INT,
    comment TEXT NOT NULL DEFAULT '',
    direction VARCHAR(8) NOT NULL,
    tx BLOB,
    FOREIGN KEY (account_id_hex) REFERENCES accounts(account_id_hex),
    FOREIGN KEY (assigned_subaddress_b58) REFERENCES assigned_subaddresses(assigned_subaddress_b58)
);
INSERT INTO NEW_transaction_logs SELECT
    id,
    transaction_id_hex,
    account_id_hex,
    assigned_subaddress_b58,
    value,
    fee,
    status,
    sent_time,
    submitted_block_index,
    finalized_block_index,
    comment,
    direction,
    tx
FROM transaction_logs;
DROP TABLE transaction_logs;
ALTER TABLE NEW_transaction_logs RENAME TO transaction_logs;
PRAGMA foreign_key_check;
PRAGMA foreign_keys=ON;
