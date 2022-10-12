-- ALTER TABLE accounts ADD COLUMN key_derivation_version INTEGER NOT NULL DEFAULT 1;
CREATE TABLE NEW_accounts (
    id INTEGER NOT NULL PRIMARY KEY,
    account_id_hex VARCHAR NOT NULL UNIQUE,
    account_key BLOB NOT NULL,
    entropy BLOB NOT NULL,
    main_subaddress_index UNSIGNED BIG INT NOT NULL,
    change_subaddress_index UNSIGNED BIG INT NOT NULL,
    next_subaddress_index UNSIGNED BIG INT NOT NULL,
    first_block_index UNSIGNED BIG INT NOT NULL,
    next_block_index UNSIGNED BIG INT NOT NULL,
    import_block_index UNSIGNED BIG INT,
    name VARCHAR NOT NULL DEFAULT '',
    key_derivation_version INTEGER NOT NULL DEFAULT 1
);
INSERT INTO NEW_accounts SELECT
    id,
    account_id_hex,
    account_key,
    entropy,
    main_subaddress_index,
    change_subaddress_index,
    next_subaddress_index,
    first_block_index,
    next_block_index,
    import_block_index,
    name,
    1
FROM accounts;
DROP TABLE accounts;
ALTER TABLE NEW_accounts RENAME TO accounts;
