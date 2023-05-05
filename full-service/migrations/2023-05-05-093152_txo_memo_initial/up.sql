CREATE TABLE txo_memos (
   txo_id varchar NOT NULL,
   memo_type SMALLINT NOT NULL,
   PRIMARY KEY  (txo_id, memo_type), 
   FOREIGN KEY (txo_id) references txos(id)
);

CREATE TABLE authenticated_sender_memos (
   txo_id VARCHAR NOT NULL,
   address_hash BLOB NOT NULL,
    /*Per mobilecoin/transaction/extra/src/memo/mod.rs : 0x0100 */
   memo_type SMALLINT generated always as (0x0100),
   PRIMARY KEY (txo_id),
   FOREIGN KEY (txo_id, memo_type) REFERENCES txo_memos(txo_id, memo_type)
);


CREATE TABLE authenticated_sender_with_payment_intent_id_memos (
   txo_id VARCHAR NOT NULL,
   address_hash BLOB NOT NULL,
   payment_intent_id BIGINT UNSIGNED NOT NULL,
   /*Per mobilecoin/transaction/extra/src/memo/mod.rs : 0x0101 */
   memo_type SMALLINT generated always as (0x0101),
   PRIMARY KEY (txo_id),
   FOREIGN KEY (txo_id, memo_type) REFERENCES txo_memos(txo_id, memo_type)
);

CREATE TABLE authenticated_sender_with_payment_request_id_memos (
   txo_id VARCHAR NOT NULL,
   address_hash BLOB NOT NULL,
   payment_request_id BIGINT UNSIGNED NOT NULL,
   /*Per mobilecoin/transaction/extra/src/memo/mod.rs : 0x0102 */
   memo_type SMALLINT generated always as (0x0102),
   PRIMARY KEY (txo_id),
   FOREIGN KEY (txo_id, memo_type) REFERENCES txo_memos(txo_id, memo_type)
);
 