-- Your SQL goes here
ALTER TABLE accounts
    ADD COLUMN spend_only_from_subaddress BOOLEAN NOT NULL DEFAULT FALSE;
