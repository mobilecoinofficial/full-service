-- Your SQL goes here
CREATE INDEX idx_txos__account_id_spent_block_index on txos (account_id,spent_block_index);