table! {
    account_txo_statuses (account_id_hex, txo_id_hex) {
        account_id_hex -> Text,
        txo_id_hex -> Text,
        txo_status -> Text,
        txo_type -> Text,
    }
}

table! {
    accounts (account_id_hex) {
        account_id_hex -> Text,
        encrypted_account_key -> Binary,
        main_subaddress_index -> BigInt,
        change_subaddress_index -> BigInt,
        next_subaddress_index -> BigInt,
        first_block -> BigInt,
        next_block -> BigInt,
        name -> Text,
    }
}

table! {
    assigned_subaddresses (assigned_subaddress_b58) {
        assigned_subaddress_b58 -> Text,
        account_id_hex -> Text,
        address_book_entry -> Nullable<BigInt>,
        public_address -> Binary,
        subaddress_index -> BigInt,
        comment -> Text,
        expected_value -> Nullable<BigInt>,
        subaddress_spend_key -> Binary,
    }
}

table! {
    transaction_logs (transaction_id_hex) {
        transaction_id_hex -> Text,
        account_id_hex -> Text,
        recipient_public_address_b58 -> Text,
        assigned_subaddress_b58 -> Text,
        value -> BigInt,
        fee -> Nullable<BigInt>,
        status -> Text,
        sent_time -> Text,
        block_height -> BigInt,
        comment -> Nullable<Text>,
        direction -> Text,
    }
}

table! {
    transactions_txos (transaction_id_hex, txo_id_hex) {
        transaction_id_hex -> Text,
        txo_id_hex -> Text,
    }
}

table! {
    txos (txo_id_hex) {
        txo_id_hex -> Text,
        value -> BigInt,
        target_key -> Binary,
        public_key -> Binary,
        e_fog_hint -> Binary,
        txo -> Binary,
        subaddress_index -> BigInt,
        key_image -> Nullable<Binary>,
        received_block_height -> Nullable<BigInt>,
        spent_tombstone_block_height -> Nullable<BigInt>,
        spent_block_height -> Nullable<BigInt>,
        proof -> Nullable<Binary>,
    }
}

joinable!(account_txo_statuses -> accounts (account_id_hex));
joinable!(account_txo_statuses -> txos (txo_id_hex));
joinable!(assigned_subaddresses -> accounts (account_id_hex));
joinable!(transaction_logs -> accounts (account_id_hex));
joinable!(transaction_logs -> assigned_subaddresses (assigned_subaddress_b58));
joinable!(transactions_txos -> transaction_logs (transaction_id_hex));
joinable!(transactions_txos -> txos (txo_id_hex));

allow_tables_to_appear_in_same_query!(
    account_txo_statuses,
    accounts,
    assigned_subaddresses,
    transaction_logs,
    transactions_txos,
    txos,
);
