table! {
    accounts (id) {
        id -> Integer,
        account_id_hex -> Text,
        account_key -> Binary,
        entropy -> Binary,
        key_derivation_version -> Integer,
        main_subaddress_index -> BigInt,
        change_subaddress_index -> BigInt,
        next_subaddress_index -> BigInt,
        first_block_index -> BigInt,
        next_block_index -> BigInt,
        import_block_index -> Nullable<BigInt>,
        name -> Text,
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
        root_entropy -> Nullable<Binary>,
        bip39_entropy -> Nullable<Binary>,
        txo_public_key -> Binary,
        value -> BigInt,
        memo -> Text,
        account_id_hex -> Text,
        txo_id_hex -> Text,
    }
}

table! {
    transaction_logs (id) {
        id -> Integer,
        transaction_id_hex -> Text,
        account_id_hex -> Text,
        assigned_subaddress_b58 -> Nullable<Text>,
        value -> BigInt,
        fee -> Nullable<BigInt>,
        status -> Text,
        sent_time -> Nullable<BigInt>,
        submitted_block_index -> Nullable<BigInt>,
        finalized_block_index -> Nullable<BigInt>,
        comment -> Text,
        direction -> Text,
        tx -> Nullable<Binary>,
    }
}

table! {
    transaction_txo_types (transaction_id_hex, txo_id_hex) {
        transaction_id_hex -> Text,
        txo_id_hex -> Text,
        transaction_txo_type -> Text,
    }
}

table! {
    txos (id) {
        id -> Integer,
        txo_id_hex -> Text,
        value -> BigInt,
        target_key -> Binary,
        public_key -> Binary,
        e_fog_hint -> Binary,
        txo -> Binary,
        subaddress_index -> Nullable<BigInt>,
        key_image -> Nullable<Binary>,
        received_block_index -> Nullable<BigInt>,
        pending_tombstone_block_index -> Nullable<BigInt>,
        spent_block_index -> Nullable<BigInt>,
        confirmation -> Nullable<Binary>,
        recipient_public_address_b58 -> Text,
        minted_account_id_hex -> Nullable<Text>,
        received_account_id_hex -> Nullable<Text>,
    }
}

allow_tables_to_appear_in_same_query!(
    accounts,
    assigned_subaddresses,
    gift_codes,
    transaction_logs,
    transaction_txo_types,
    txos,
);
