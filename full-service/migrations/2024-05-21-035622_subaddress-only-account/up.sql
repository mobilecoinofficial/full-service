-- Your SQL goes here
ALTER TABLE accounts
    ADD COLUMN require_spend_subaddresses BOOLEAN NOT NULL DEFAULT FALSE;