table! {
    accounts (account_id_hex) {
        account_id_hex -> Text,
        encrypted_account_key -> Binary,
        main_subaddress_index -> Text,
        change_subaddress_index -> Text,
        next_subaddress_index -> Text,
        first_block -> Text,
        next_block -> Text,
        name -> Nullable<Text>,
    }
}
