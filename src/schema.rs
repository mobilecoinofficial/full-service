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
