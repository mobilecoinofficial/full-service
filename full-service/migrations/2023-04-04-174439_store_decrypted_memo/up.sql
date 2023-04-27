CREATE TABLE supported_memo_types (
  classifier int PRIMARY KEY NOT NULL,
  memo_type_name varchar(50) NOT NULL unique
);

CREATE TABLE txo_memos (
  txo_id varchar NOT NULL PRIMARY KEY,
  memo_type int NOT NULL,
  FOREIGN KEY (txo_id) references txos(id)
  constraint memo_type_fk FOREIGN KEY(memo_type) references supported_memo_types(classifier)
);


CREATE TABLE authenticated_sender_memos (
   txo_id VARCHAR NOT NULL,
   address_hash BLOB NOT NULL,
   memo_type integer generated always as (1),
   PRIMARY KEY (txo_id),m
   FOREIGN KEY (memo_type, txo_id) REFERENCES txo_memos(memo_type, txo_id)
);


CREATE TABLE authenticated_sender_with_payment_intent_id_memos (
   txo_id VARCHAR NOT NULL,
   address_hash BLOB NOT NULL,
   payment_intent_id BLOB NOT NULL,
      memo_type integer generated always as (2),
   PRIMARY KEY (txo_id),
   FOREIGN KEY (memo_type, txo_id) REFERENCES txo_memos(memo_type, txo_id)
);

CREATE TABLE authenticated_sender_with_payment_request_id_memos (
   txo_id VARCHAR NOT NULL,
   address_hash BLOB NOT NULL,
   payment_request_id BLOB NOT NULL,
      memo_type integer generated always as (3),
   PRIMARY KEY (txo_id),
   FOREIGN KEY (memo_type, txo_id) REFERENCES txo_memos(memo_type, txo_id)
);
