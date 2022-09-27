CREATE TABLE NEW_gift_codes (
  id INTEGER NOT NULL PRIMARY KEY,
  gift_code_b58 VARCHAR NOT NULL,
  value UNSIGNED BIG INT NOT NULL
);
INSERT INTO NEW_gift_codes SELECT id, gift_code_b58, value FROM gift_codes;
DROP TABLE gift_codes;
ALTER TABLE NEW_gift_codes RENAME TO gift_codes;

