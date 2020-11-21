// Copyright (c) 2020 MobileCoin Inc.

use crate::db::WalletDb;
use diesel::{prelude::*, SqliteConnection};
use diesel_migrations::embed_migrations;
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
