-- Your SQL goes here
ALTER TABLE accounts
    ADD COLUMN require_spend_subaddress BOOLEAN NOT NULL DEFAULT FALSE;