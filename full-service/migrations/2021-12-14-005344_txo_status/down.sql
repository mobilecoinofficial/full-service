CREATE TABLE account_txo_statuses (
      account_id_hex TEXT NOT NULL,
      txo_id_hex TEXT NOT NULL,
      txo_status TEXT NOT NULL,
      txo_type TEXT NOT NULL,
      PRIMARY KEY (account_id_hex, txo_id_hex),
      FOREIGN KEY (account_id_hex) REFERENCES accounts(account_id_hex),
      FOREIGN KEY (txo_id_hex) REFERENCES txos(txo_id_hex)
);

-- Minted txo, not received, or received by a different account.
INSERT INTO account_txo_statuses (
    account_id_hex,
    txo_id_hex,
    txo_status,
    txo_type
)
SELECT
    minted_account_id_hex,
    txo_id_hex,
    'txo_status_secreted',
    'txo_type_minted'
FROM txos
WHERE minted_account_id_hex IS NOT NULL
    AND received_account_id_hex != minted_account_id_hex;

-- Received txo.
INSERT INTO account_txo_statuses (
    account_id_hex,
    txo_id_hex,
    txo_status,
    txo_type
)
SELECT
    received_account_id_hex,
    txo_id_hex,
    CASE
        WHEN spent_block_index IS NOT NULL
            THEN 'txo_status_spent'
        ELSE
            CASE
                WHEN pending_tombstone_block_index IS NOT NULL
                    THEN 'txo_status_pending'
                ELSE
                    CASE
                        WHEN subaddress_index IS NULL
                            THEN 'txo_status_orphaned'
                        ELSE 'txo_status_unspent'
                    END
            END
    END,
    'txo_type_received'
FROM txos
WHERE received_account_id_hex IS NOT NULL;

ALTER TABLE txos DROP COLUMN minted_account_id_hex;
ALTER TABLE txos DROP COLUMN received_account_id_hex;
