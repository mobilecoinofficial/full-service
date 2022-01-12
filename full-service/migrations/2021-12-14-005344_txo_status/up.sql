ALTER TABLE txos ADD COLUMN minted_account_id_hex TEXT NULL;
ALTER TABLE txos ADD COLUMN received_account_id_hex TEXT NULL;

UPDATE txos
SET minted_account_id_hex = account_id_hex
FROM (
    SELECT txo_id_hex, account_id_hex
    FROM account_txo_statuses
    WHERE txo_type='txo_type_minted'
) as status
WHERE txos.txo_id_hex = status.txo_id_hex;

UPDATE txos
SET received_account_id_hex = account_id_hex
FROM (
    SELECT txo_id_hex, account_id_hex
    FROM account_txo_statuses
    WHERE txo_type='txo_type_received'
) as status
WHERE txos.txo_id_hex = status.txo_id_hex;

DROP TABLE account_txo_statuses;
