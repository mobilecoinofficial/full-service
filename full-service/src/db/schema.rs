table! {
  accounts (id) {
      id -> Text,
      account_key -> Binary,
      entropy -> Nullable<Binary>,
      key_derivation_version -> Integer,
      first_block_index -> BigInt,
      next_block_index -> BigInt,
      import_block_index -> Nullable<BigInt>,
      name -> Text,
      fog_enabled -> Bool,
      view_only -> Bool,
  }
}

table! {
  assigned_subaddresses (public_address_b58) {
      public_address_b58 -> Text,
      account_id -> Text,
      subaddress_index -> BigInt,
      comment -> Text,
      spend_public_key -> Binary,
  }
}

table! {
  authenticated_sender_memos (txo_id) {
      txo_id -> Text,
      address_hash -> Binary,
      memo_type -> Nullable<Integer>,
  }
}

table! {
  authenticated_sender_with_payment_intent_id_memos (txo_id) {
      txo_id -> Text,
      address_hash -> Binary,
      payment_intent_id -> Binary,
      memo_type -> Nullable<Integer>,
  }
}

table! {
  authenticated_sender_with_payment_request_id_memos (txo_id) {
      txo_id -> Text,
      address_hash -> Binary,
      payment_request_id -> Binary,
      memo_type -> Nullable<Integer>,
  }
}

table! {
  gift_codes (id) {
      id -> Integer,
      gift_code_b58 -> Text,
      value -> BigInt,
  }
}

table! {
  supported_memo_types (classifier) {
      classifier -> Integer,
      memo_type_name -> Text,
  }
}

table! {
  transaction_input_txos (transaction_log_id, txo_id) {
      transaction_log_id -> Text,
      txo_id -> Text,
  }
}

table! {
  transaction_logs (id) {
      id -> Text,
      account_id -> Text,
      fee_value -> BigInt,
      fee_token_id -> BigInt,
      submitted_block_index -> Nullable<BigInt>,
      tombstone_block_index -> Nullable<BigInt>,
      finalized_block_index -> Nullable<BigInt>,
      comment -> Text,
      tx -> Binary,
      failed -> Bool,
  }
}

table! {
  transaction_output_txos (transaction_log_id, txo_id) {
      transaction_log_id -> Text,
      txo_id -> Text,
      recipient_public_address_b58 -> Text,
      is_change -> Bool,
  }
}

table! {
  txo_memos (txo_id) {
      txo_id -> Text,
      memo_type -> Integer,
  }
}

table! {
  txos (id) {
      id -> Text,
      account_id -> Nullable<Text>,
      value -> BigInt,
      token_id -> BigInt,
      target_key -> Binary,
      public_key -> Binary,
      e_fog_hint -> Binary,
      subaddress_index -> Nullable<BigInt>,
      key_image -> Nullable<Binary>,
      received_block_index -> Nullable<BigInt>,
      spent_block_index -> Nullable<BigInt>,
      confirmation -> Nullable<Binary>,
      shared_secret -> Nullable<Binary>,
  }
}

table! {
  __diesel_schema_migrations(version) {
      version -> Text,
      run_on -> Timestamp,
  }
}

joinable!(assigned_subaddresses -> accounts (account_id));
joinable!(transaction_input_txos -> transaction_logs (transaction_log_id));
joinable!(transaction_input_txos -> txos (txo_id));
joinable!(transaction_logs -> accounts (account_id));
joinable!(transaction_output_txos -> transaction_logs (transaction_log_id));
joinable!(transaction_output_txos -> txos (txo_id));
joinable!(txo_memos -> supported_memo_types (memo_type));
joinable!(txo_memos -> txos (txo_id));
joinable!(txos -> accounts (account_id));

allow_tables_to_appear_in_same_query!(
    accounts,
    assigned_subaddresses,
    authenticated_sender_memos,
    authenticated_sender_with_payment_intent_id_memos,
    authenticated_sender_with_payment_request_id_memos,
    gift_codes,
    supported_memo_types,
    transaction_input_txos,
    transaction_logs,
    transaction_output_txos,
    txo_memos,
    txos,
);
