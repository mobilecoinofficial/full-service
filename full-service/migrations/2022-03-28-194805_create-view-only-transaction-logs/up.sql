-- Your SQL goes here
CREATE TABLE view_only_transaction_logs (
    id INTEGER NOT NULL PRIMARY KEY,
    change_txo_id_hex TEXT NOT NULL,
    input_txo_id_hex TEXT NOT NULL
);
