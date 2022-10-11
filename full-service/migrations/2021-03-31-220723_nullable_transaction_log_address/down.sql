-- ALTER TABLE transaction_logs ALTER COLUMN assigned_subaddress_b58 SET NOT NULL;
CREATE TABLE OLD_transaction_logs (
    id INTEGER NOT NULL PRIMARY KEY,
    transaction_id_hex VARCHAR NOT NULL UNIQUE,
    account_id_hex VARCHAR NOT NULL,
    recipient_public_address_b58 VARCHAR NOT NULL DEFAULT '',
    assigned_subaddress_b58 VARCHAR NOT NULL DEFAULT '',
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
INSERT INTO OLD_transaction_logs SELECT * FROM transaction_logs;
DROP TABLE transaction_logs;
ALTER TABLE OLD_transaction_logs RENAME TO transaction_logs;
