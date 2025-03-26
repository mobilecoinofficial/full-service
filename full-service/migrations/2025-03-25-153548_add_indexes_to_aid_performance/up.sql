-- below speeds up ledger-db-> wallet-db sync performance by 2x on an account with 50k tlogs + 50k txos in a wallet-db with ~1M txos
CREATE INDEX idx_txos__account_id_spent_block_index on txos (account_id,spent_block_index);
-- below speeds up sync performance by 1.5~2x
CREATE INDEX idx_transaction_logs__account_id_finalized_block_index on transaction_logs (account_id, finalized_block_index);
/* 
  The two indexes below speed up deleting an account. For a reference account with no tlogs where the
  wallet had 1m txos and 50k tlogs the speedup was from >6 hours to 30 seconds.
  Also, the wallet is locked during the delete operation, so taking hours is problematic.
  These indexes should also benefit joins between indexed tables and the txos table.
*/
CREATE INDEX idx_transaction_input_txos__txo_id on transaction_input_txos (txo_id);
CREATE INDEX idx_transaction_output_txos__txo_id on transaction_output_txos (txo_id);
