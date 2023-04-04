-- This file should undo anything in `up.sql`
ALTER TABLE txos DROP COLUMN memo;
ALTER TABLE txos DROP COLUMN memo_type;
ALTER TABLE txos DROP COLUMN address_hash;
