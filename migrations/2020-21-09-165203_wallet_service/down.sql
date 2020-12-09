drop TABLE transaction_txo_types;

drop INDEX idx_transaction_logs__transaction_id_hex;
drop TABLE transaction_logs;

DROP INDEX idx_assigned_subaddresses__assigned_subaddress_b58;
DROP TABLE assigned_subaddresses;

DROP TABLE account_txo_statuses;

DROP INDEX idx_txos__txo_id_hex;
DROP TABLE txos;

DROP INDEX idx_accounts__account_id_hex;
DROP TABLE accounts;
