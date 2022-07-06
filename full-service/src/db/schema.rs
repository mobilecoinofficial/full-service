table! {
    accounts (id) {
        id -> Integer,
        account_id_hex -> Text,
        account_key -> Binary,
        entropy -> Nullable<Binary>,
        key_derivation_version -> Integer,
        main_subaddress_index -> BigInt,
        change_subaddress_index -> BigInt,
        next_subaddress_index -> BigInt,
        first_block_index -> BigInt,
        next_block_index -> BigInt,
        import_block_index -> Nullable<BigInt>,
        name -> Text,
        fog_enabled -> Bool,
        view_only -> Bool,
    }
}

table! {
    assigned_subaddresses (id) {
        id -> Integer,
        assigned_subaddress_b58 -> Text,
        account_id_hex -> Text,
        address_book_entry -> Nullable<BigInt>,
        public_address -> Binary,
        subaddress_index -> BigInt,
        comment -> Text,
        subaddress_spend_key -> Binary,
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
    transaction_change_txos (transaction_log_id, txo_id) {
        transaction_log_id -> Text,
        txo_id -> Text,
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
        account_id_hex -> Text,
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
    transaction_payload_txos (transaction_log_id, txo_id) {
        transaction_log_id -> Text,
        txo_id -> Text,
        recipient_public_address_b58 -> Text,
    }
}

table! {
    txos (id) {
        id -> Text,
        account_id_hex -> Nullable<Text>,
        value -> BigInt,
        token_id -> BigInt,
        target_key -> Binary,
        public_key -> Binary,
        e_fog_hint -> Binary,
        txo -> Binary,
        subaddress_index -> Nullable<BigInt>,
        key_image -> Nullable<Binary>,
        received_block_index -> Nullable<BigInt>,
        spent_block_index -> Nullable<BigInt>,
        shared_secret -> Nullable<Binary>,
    }
}

joinable!(transaction_change_txos -> transaction_logs (transaction_log_id));
joinable!(transaction_change_txos -> txos (txo_id));
joinable!(transaction_input_txos -> transaction_logs (transaction_log_id));
joinable!(transaction_input_txos -> txos (txo_id));
joinable!(transaction_payload_txos -> transaction_logs (transaction_log_id));
joinable!(transaction_payload_txos -> txos (txo_id));

allow_tables_to_appear_in_same_query!(
    accounts,
    assigned_subaddresses,
    gift_codes,
    transaction_change_txos,
    transaction_input_txos,
    transaction_logs,
    transaction_payload_txos,
    txos,
);
