ALTER TABLE accounts ADD COLUMN resyncing BOOLEAN NOT NULL DEFAULT FALSE;

UPDATE accounts SET next_block_index = 0;
UPDATE accounts SET resyncing = TRUE;