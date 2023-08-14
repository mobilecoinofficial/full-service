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
        sender_address_hash -> Text,
        payment_request_id -> Nullable<Text>,
        payment_intent_id -> Nullable<Text>,
        validated -> Bool,
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
        memo_type -> Nullable<Integer>,
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
joinable!(txos -> accounts (account_id));

allow_tables_to_appear_in_same_query!(
    accounts,
    assigned_subaddresses,
    gift_codes,
    transaction_input_txos,
    transaction_logs,
    transaction_output_txos,
    txos,
);
