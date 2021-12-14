-- todo: add accounttxostatus
-- todo: data migration
ALTER TABLE txos REMOVE COLUMN minted_account_id_hex;
ALTER TABLE txos REMOVE COLUMN received_account_id_hex;

