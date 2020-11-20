// Copyright (c) 2020 MobileCoin Inc.

//! Provides the CRUD implementations for our DB, and converts types to what is expected
//! by the DB.

use mc_account_keys::{AccountKey, PublicAddress};
use mc_crypto_digestible::{Digestible, MerlinTranscript};

use diesel::prelude::*;
use diesel::Connection;
use diesel::RunQueryDsl;
use dotenv::dotenv;
use std::env;

use crate::error::WalletDBError;
use crate::models::*;
use crate::schema::accounts;

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn create_account(
    conn: &SqliteConnection,
    account_key: &AccountKey,
    main_subaddress_index: u64,
    change_subaddress_index: u64,
    next_subaddress_index: u64,
    first_block: u64,
    next_block: u64,
    name: &str,
) -> Result<String, WalletDBError> {
    // Get the account ID from the contents of the account key
    #[derive(Digestible)]
    struct ConstAccountData {
        // We use PublicAddress and not AccountKey so that the monitor_id is not sensitive.
        pub address: PublicAddress,
        pub main_subaddress: u64,
        pub first_block: u64,
    }
    let const_data = ConstAccountData {
        address: account_key.subaddress(main_subaddress_index),
        main_subaddress: main_subaddress_index,
        first_block: first_block,
    };
    let temp: [u8; 32] = const_data.digest32::<MerlinTranscript>(b"monitor_data");
    let account_id_hex = hex::encode(temp);

    // FIXME: how do we want to do optional/defaults and overrides?
    let new_account = NewAccount {
        account_id_hex: &account_id_hex,
        encrypted_account_key: &mc_util_serial::encode(account_key), // FIXME: add encryption
        main_subaddress_index: &main_subaddress_index.to_string(),
        change_subaddress_index: &change_subaddress_index.to_string(),
        next_subaddress_index: &next_subaddress_index.to_string(),
        first_block: &first_block.to_string(),
        next_block: &next_block.to_string(),
        name,
    };

    diesel::insert_into(accounts::table)
        .values(&new_account)
        .execute(conn)?;

    Ok(account_id_hex)
}

/* Example UPDATE
pub fn publish_post(conn: &SqliteConnection, key: String) -> usize {
    diesel::update(dsl_accounts.find(key))
        .set(published.eq(true))
        .execute(conn)
        .expect("Unable to find post")
}
 */
