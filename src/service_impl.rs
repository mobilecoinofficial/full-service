// Copyright (c) 2020 MobileCoin Inc.

//! The implementation of the wallet service methods
use crate::db::create_account;
// use mc_account_keys::{AccountKey, PublicAddress, RootIdentity, DEFAULT_SUBADDRESS_INDEX};
// use mc_crypto_digestible::{Digestible, MerlinTranscript};

use diesel::prelude::*;

pub const DEFAULT_CHANGE_SUBADDRESS_INDEX: u64 = 1;
pub const DEFAULT_NEXT_SUBADDRESS_INDEX: u64 = 2;
pub const DEFAULT_FIRST_BLOCK: u64 = 0;

pub fn create_account_impl(conn: &SqliteConnection, name: String) -> (String, String, String) {
    // Generate entropy for the account
    // let mut rng = rand::thread_rng();
    // let root_id = RootIdentity::from_random(&mut rng);
    // let entropy_str = hex::encode(root_id.root_entropy);
    let entropy_str = "Test";
    let account_key: Vec<u8> = Vec::new();
    // let account_key = AccountKey::from(&root_id);
    // let public_address = account_key.subaddress(DEFAULT_SUBADDRESS_INDEX);
    // FIXME: printable wrapper for public address

    // #[derive(Digestible)]
    // struct ConstAccountData {
    //     // We use PublicAddress and not AccountKey so that the monitor_id is not sensitive.
    //     pub address: PublicAddress,
    //     pub main_subaddress: u64,
    //     pub first_block: u64,
    // }
    // let const_data = ConstAccountData {
    //     address: public_address,
    //     main_subaddress: DEFAULT_SUBADDRESS_INDEX,
    //     first_block: DEFAULT_FIRST_BLOCK,
    // };
    // let temp: [u8; 32] = const_data.digest32::<MerlinTranscript>(b"monitor_data");
    // let account_id = hex::encode(temp);
    let account_id = "test";

    create_account(
        conn,
        account_id,
        &account_key,
        &0.to_string(), //DEFAULT_SUBADDRESS_INDEX,
        &DEFAULT_CHANGE_SUBADDRESS_INDEX.to_string(),
        &DEFAULT_NEXT_SUBADDRESS_INDEX.to_string(),
        &DEFAULT_FIRST_BLOCK.to_string(),
        &(DEFAULT_FIRST_BLOCK + 1).to_string(),
        &name,
    );

    (
        entropy_str.to_string(),
        "b58_public_address".to_string(),
        account_id.to_string(),
    )
}
