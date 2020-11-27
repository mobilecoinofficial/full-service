// Copyright (c) 2020 MobileCoin Inc.

use crate::db::WalletDb;
use diesel::{prelude::*, SqliteConnection};
use diesel_migrations::embed_migrations;
// use mc_account_keys::PublicAddress;
// use mc_crypto_rand::{CryptoRng, RngCore};
use rand::{distributions::Alphanumeric, thread_rng, Rng};

embed_migrations!("migrations/");

pub struct WalletDbTestContext {
    base_url: String,
    pub db_name: String,
}

impl Default for WalletDbTestContext {
    fn default() -> Self {
        dotenv::dotenv().unwrap();

        let db_name: String = format!(
            "test_{}",
            thread_rng()
                .sample_iter(&Alphanumeric)
                .take(10)
                .collect::<String>()
                .to_lowercase()
        );
        let base_url = std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");

        // Connect to the database and run the migrations
        let conn = SqliteConnection::establish(&format!("{}/{}", base_url, db_name))
            .unwrap_or_else(|err| panic!("Cannot connect to {} database: {:?}", db_name, err));

        embedded_migrations::run(&conn).expect("failed running migrations");

        // Success
        Self { base_url, db_name }
    }
}

impl WalletDbTestContext {
    pub fn get_db_instance(&self) -> WalletDb {
        WalletDb::new_from_url(&format!("{}/{}", self.base_url, self.db_name))
            .expect("failed creating new SqlRecoveryDb")
    }
}

/*
/// Sets up ledger_db. Each block contains one txo per recipient.
///
/// # Arguments
/// *
/// * `num_random_recipients` - Number of random recipients to create.
/// * `known_recipients` - A list of known recipients to create.
/// * `num_blocks` - Number of blocks to create in the ledger_db.
/// * `rng`
///
/// Note that all txos will be controlled by the subindex DEFAULT_SUBADDRESS_INDEX
pub fn get_test_ledger(
    num_random_recipients: u32,
    known_recipients: &[PublicAddress],
    num_blocks: usize,
    mut rng: &mut (impl CryptoRng + RngCore),
) -> LedgerDB {
    let mut public_addresses: Vec<PublicAddress> = (0..num_random_recipients)
        .map(|_i| mc_account_keys::AccountKey::random(&mut rng).default_subaddress())
        .collect();

    public_addresses.extend(known_recipients.iter().cloned());

    // Note that TempDir manages uniqueness by constructing paths
    // like: /tmp/ledger_db.tvF0XHTKsilx
    let ledger_db_tmp = TempDir::new("ledger_db").expect("Could not make tempdir for ledger db");
    let ledger_db_path = ledger_db_tmp
        .path()
        .to_str()
        .expect("Could not get path as string");

    let mut ledger_db = generate_ledger_db(&ledger_db_path);

    for block_index in 0..num_blocks {
        let key_images = if block_index == 0 {
            vec![]
        } else {
            vec![KeyImage::from(rng.next_u64())]
        };
        let _new_block_height = add_block_to_ledger_db(
            &mut ledger_db,
            &public_addresses,
            DEFAULT_PER_RECIPIENT_AMOUNT,
            &key_images,
            rng,
        );
    }

    ledger_db
}

*/
