// Copyright (c) 2020 MobileCoin Inc.

table! {
    account_txo_statuses (account_id_hex, txo_id_hex) {
        account_id_hex -> Text,
        txo_id_hex -> Text,
        txo_status -> Text,
        txo_type -> Text,
    }
}

table! {
    accounts (id) {
        id -> Integer,
        account_id_hex -> Text,
        encrypted_account_key -> Binary,
        main_subaddress_index -> BigInt,
        change_subaddress_index -> BigInt,
        next_subaddress_index -> BigInt,
        first_block -> BigInt,
        next_block -> BigInt,
        import_block -> Nullable<BigInt>,
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
    transaction_logs (id) {
        id -> Integer,
        transaction_id_hex -> Text,
        account_id_hex -> Text,
        recipient_public_address_b58 -> Text,
        assigned_subaddress_b58 -> Text,
        value -> BigInt,
        fee -> Nullable<BigInt>,
        status -> Text,
        sent_time -> Nullable<BigInt>,
        submitted_block_count -> Nullable<BigInt>,
        finalized_block_count -> Nullable<BigInt>,
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
        received_block_count -> Nullable<BigInt>,
        pending_tombstone_block_count -> Nullable<BigInt>,
        spent_block_count -> Nullable<BigInt>,
        proof -> Nullable<Binary>,
    }
}

allow_tables_to_appear_in_same_query!(
    account_txo_statuses,
    accounts,
    assigned_subaddresses,
    transaction_logs,
    transaction_txo_types,
    txos,
);
