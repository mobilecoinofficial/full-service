-- Your SQL goes here
ALTER TABLE txos ADD COLUMN memo BLOB NOT NULL DEFAULT '';
ALTER TABLE txos ADD COLUMN memo_type INTEGER NULL;
ALTER TABLE txos ADD COLUMN address_hash BLOB NULL;
CREATE INDEX idx_txos_memo_type ON txos (memo_type);
CREATE INDEX idx_txos_address_hash ON txos (address_hash);
