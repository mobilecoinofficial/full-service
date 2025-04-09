-- This file should undo anything in `up.sql`
DROP INDEX idx_transaction_output_txos__txo_id;
DROP INDEX idx_transaction_input_txos__txo_id;
DROP INDEX idx_transaction_logs__account_id_finalized_block_index;
DROP INDEX idx_txos__account_id_spent_block_index;
