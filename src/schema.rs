table! {
    account_txo_status (account_id_hex, txo_id_hex) {
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
    txos (txo_id_hex) {
        txo_id_hex -> Text,
        value -> BigInt,
        target_key -> Binary,
        public_key -> Binary,
        e_fog_hint -> Binary,
        subaddress_index -> BigInt,
        key_image -> Nullable<Binary>,
        received_block_height -> Nullable<BigInt>,
        spent_tombstone_block_height -> Nullable<BigInt>,
        spent_block_height -> Nullable<BigInt>,
        proof -> Nullable<Binary>,
    }
}

joinable!(account_txo_status -> accounts (account_id_hex));
joinable!(account_txo_status -> txos (txo_id_hex));

allow_tables_to_appear_in_same_query!(account_txo_status, accounts, txos,);
