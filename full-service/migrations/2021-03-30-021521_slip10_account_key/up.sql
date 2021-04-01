ALTER TABLE accounts
ADD COLUMN key_derivation_version INTEGER NOT NULL DEFAULT 1;
