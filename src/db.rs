// Copyright (c) 2020 MobileCoin Inc.

//! Provides the CRUD implementations for our DB

use diesel::prelude::*;
use diesel::Connection;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use dotenv::dotenv;
use std::env;

// use accounts::dsl::accounts as dsl_accounts;
use crate::models::*;
use crate::schema::accounts;

use uuid::Uuid;

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn create_account(
    conn: &SqliteConnection,
    account_id_hex: &str,
    encrypted_account_key: &Vec<u8>,
    main_subaddress_index: &str,
    change_subaddress_index: &str,
    next_subaddress_index: &str,
    first_block: &str,
    next_block: &str,
    name: &str,
) {
    // FIXME: should these be typed?
    // FIXME: how do we do optional/defaults and overrides?
    let new_account = NewAccount {
        account_id_hex,
        encrypted_account_key,
        main_subaddress_index,
        change_subaddress_index,
        next_subaddress_index,
        first_block,
        next_block,
        name,
    };

    diesel::insert_into(accounts::table)
        .values(&new_account)
        .execute(conn)
        .expect("Error saving new post");
}

/* Example UPDATE
pub fn publish_post(conn: &SqliteConnection, key: String) -> usize {
    diesel::update(dsl_accounts.find(key))
        .set(published.eq(true))
        .execute(conn)
        .expect("Unable to find post")
}
 */
