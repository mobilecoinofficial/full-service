use crate::db::{
    models::{AssignedSubaddress, Migration, NewMigration, PostMigrationProcess, Txo},
    schema::{__diesel_schema_migrations, assigned_subaddresses, post_migration_processes, txos},
    txo::TxoModel,
    WalletDbError,
};
use diesel::{
    connection::SimpleConnection,
    prelude::*,
    r2d2::{ConnectionManager, Pool, PooledConnection},
    sql_types,
    sqlite::Sqlite,
    SqliteConnection,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use mc_common::logger::{global_log, log, Logger};
use mc_crypto_keys::RistrettoPublic;
use std::{env, thread::sleep, time::Duration};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

pub type Conn<'a> = &'a mut SqliteConnection;

#[derive(Debug)]
pub struct ConnectionOptions {
    pub enable_wal: bool,
    pub enable_foreign_keys: bool,
    pub busy_timeout: Option<Duration>,
}

impl diesel::r2d2::CustomizeConnection<SqliteConnection, diesel::r2d2::Error>
    for ConnectionOptions
{
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), diesel::r2d2::Error> {
        (|| {
            if let Some(d) = self.busy_timeout {
                conn.batch_execute(&format!("PRAGMA busy_timeout = {};", d.as_millis()))?;
            }
            WalletDb::set_db_encryption_key_from_env(conn);
            if self.enable_wal {
                conn.batch_execute("
                    PRAGMA journal_mode = WAL;          -- better write-concurrency
                    PRAGMA synchronous = NORMAL;        -- fsync only in critical moments
                    PRAGMA wal_autocheckpoint = 1000;   -- write WAL changes back every 1000 pages, for an in average 1MB WAL file. May affect readers if number is increased
                    PRAGMA wal_checkpoint(TRUNCATE);    -- free some space by truncating possibly massive WAL files from the last run.
                ")?;
            }
            if self.enable_foreign_keys {
                conn.batch_execute("PRAGMA foreign_keys = ON;")?;
            } else {
                conn.batch_execute("PRAGMA foreign_keys = OFF;")?;
            }

            Ok(())
        })()
        .map_err(diesel::r2d2::Error::QueryError)
    }
}

#[derive(Clone)]
pub struct WalletDb {
    pub pool: Pool<ConnectionManager<SqliteConnection>>,
}

impl WalletDb {
    pub fn new(pool: Pool<ConnectionManager<SqliteConnection>>) -> Self {
        Self { pool }
    }

    pub fn new_from_url(database_url: &str, db_connections: u32) -> Result<Self, WalletDbError> {
        let manager = ConnectionManager::<SqliteConnection>::new(database_url);
        let pool = Pool::builder()
            .max_size(db_connections)
            .connection_customizer(Box::new(ConnectionOptions {
                enable_wal: true,
                enable_foreign_keys: true,
                busy_timeout: Some(Duration::from_secs(30)),
            }))
            .test_on_check_out(true)
            .build(manager)?;
        Ok(Self::new(pool))
    }

    pub fn get_pooled_conn(
        &self,
    ) -> Result<PooledConnection<ConnectionManager<SqliteConnection>>, WalletDbError> {
        Ok(self.pool.get()?)
    }

    pub fn set_db_encryption_key_from_env(conn: &mut SqliteConnection) {
        // Send the encryption key to SQLCipher, if it is not the empty string.
        let encryption_key = env::var("MC_PASSWORD").unwrap_or_else(|_| "".to_string());
        if !encryption_key.is_empty() {
            let result = conn.batch_execute(&format!(
                "PRAGMA key = {};",
                sql_escape_string(&encryption_key)
            ));
            if result.is_err() {
                panic!("Could not decrypt database.");
            }
        }
    }

    pub fn try_change_db_encryption_key_from_env(conn: &mut SqliteConnection) {
        // Change the encryption key if specified by the environment variable.
        let encryption_key = env::var("MC_PASSWORD").unwrap_or_else(|_| "".to_string());
        let changed_encryption_key =
            env::var("MC_CHANGED_PASSWORD").unwrap_or_else(|_| "".to_string());
        if !encryption_key.is_empty()
            && !changed_encryption_key.is_empty()
            && encryption_key != changed_encryption_key
        {
            let result = conn.batch_execute(&format!(
                "PRAGMA rekey = {};",
                sql_escape_string(&changed_encryption_key)
            ));
            if result.is_err() {
                panic!("Could not set new password.");
            }
            // Set the new password in the environment, so other threads can decrypt
            // correctly.
            env::set_var("MC_PASSWORD", changed_encryption_key);
            global_log::info!("Re-encrypted database with new password.");
        }
    }

    pub fn check_database_connectivity(conn: &mut SqliteConnection) -> bool {
        conn.batch_execute("SELECT count(*) FROM sqlite_master;")
            .is_ok()
    }

    pub fn validate_foreign_keys(conn: &mut SqliteConnection) {
        let invalid_foreign_keys = diesel::dsl::sql::<(
            sql_types::Text,
            sql_types::Int8,
            sql_types::Text,
            sql_types::Int8,
        )>("PRAGMA foreign_key_check;")
        .get_results::<(String, i64, String, i64)>(conn)
        .expect("failed querying for invalid foreign keys");

        if !invalid_foreign_keys.is_empty() {
            panic!(
                "Database contains invalid foreign keys: {:?}",
                invalid_foreign_keys
            );
        }
    }

    // check for and retroactively insert any missing migrations if there is a later
    // migration without the prior ones.
    // We need to perform this first check in case this is a fresh database, in
    // which case there will be no migrations table.
    pub fn add_mising_migrations(conn: Conn) {
        if let Ok(migrations) = __diesel_schema_migrations::table.load::<Migration>(conn) {
            global_log::debug!("Number of migrations applied: {:?}", migrations.len());

            if migrations.len() == 1 && migrations[0].version == "20220613204000" {
                global_log::debug!("Retroactively inserting missing migrations");
                let missing_migrations = vec![
                    NewMigration::new("20202109165203"),
                    NewMigration::new("20210303035127"),
                    NewMigration::new("20210307192850"),
                    NewMigration::new("20210308031049"),
                    NewMigration::new("20210325042338"),
                    NewMigration::new("20210330021521"),
                    NewMigration::new("20210331220723"),
                    NewMigration::new("20210403183001"),
                    NewMigration::new("20210409050201"),
                    NewMigration::new("20210420182449"),
                    NewMigration::new("20210625225113"),
                    NewMigration::new("20211214005344"),
                    NewMigration::new("20220208225206"),
                    NewMigration::new("20220215200456"),
                    NewMigration::new("20220228190052"),
                    NewMigration::new("20220328194805"),
                    NewMigration::new("20220427170453"),
                    NewMigration::new("20220513170243"),
                    NewMigration::new("20220601162825"),
                ];

                diesel::insert_into(__diesel_schema_migrations::table)
                    .values(&missing_migrations)
                    .execute(conn)
                    .expect("failed inserting migration");
            }
        }
    }

    pub fn run_migrations(conn: &mut impl MigrationHarness<Sqlite>) {
        conn.run_pending_migrations(MIGRATIONS)
            .expect("failed running migrations");
    }

    pub fn run_post_migration_processes(logger: &Logger, conn: &mut SqliteConnection) {
        let pending_post_migration_processes: Vec<PostMigrationProcess> =
            post_migration_processes::table
                .filter(post_migration_processes::has_run.eq(false))
                .load::<PostMigrationProcess>(conn)
                .expect("failed querying for pending post migration processes");

        for pending_post_migration_process in pending_post_migration_processes {
            match pending_post_migration_process.migration_version.as_str() {
                "20230814214222" => {
                    Self::run_post_migration_process_version_20230814214222(logger, conn);
                }
                _ => panic!(
                    "Unknown post migration process version: {}",
                    pending_post_migration_process.migration_version
                ),
            }
        }
    }

    /// This post migration process rescans all txos in the the database and
    /// decodes/stores any AuthenticatedSenderMemos (and vairants of)
    /// that are found.
    fn run_post_migration_process_version_20230814214222(
        logger: &Logger,
        conn: &mut SqliteConnection,
    ) {
        log::info!(
            logger,
            "Running post migration process version 20230814214222"
        );

        let txos = txos::table
            .load::<Txo>(conn)
            .expect("failed querying for txos");

        diesel::update(post_migration_processes::table)
            .filter(post_migration_processes::migration_version.eq("20230814214222"))
            .set(post_migration_processes::has_run.eq(true))
            .execute(conn)
            .expect("failed updating post migration process");
    }

    pub fn run_proto_conversions_if_necessary(conn: &mut SqliteConnection) {
        Self::run_assigned_subaddress_proto_conversions(conn);
    }

    /// Prior to v2.0.0, the spend public key of a subaddress was stored as
    /// protobuf bytes unnecessarily. This converts those to raw bytes instead,
    /// which is what users will most likely be expecting.
    ///
    /// This is a one-time conversion, so we check if the conversion has already
    /// happened, and if it has we do nothing.
    fn run_assigned_subaddress_proto_conversions(conn: &mut SqliteConnection) {
        global_log::debug!("Checking for assigned subaddress proto conversions");
        let assigned_subaddresses = assigned_subaddresses::table
            .load::<AssignedSubaddress>(conn)
            .expect("failed querying for assigned subaddresses");

        for assigned_subaddress in assigned_subaddresses {
            // Checking if the data is encoded as protobuf bytes, and if it is, we turn it
            // into raw bytes instead.
            //
            // If the spend public key is already raw bytes, we can assume the rest of the
            // subaddresses are too, so we can return early.
            let spend_public_key_bytes = match mc_util_serial::decode::<RistrettoPublic>(
                &assigned_subaddress.spend_public_key,
            ) {
                Ok(spend_public_key) => spend_public_key.to_bytes().to_vec(),
                Err(_) => {
                    global_log::debug!(
                        "Assigned subaddress proto conversion already done, skipping..."
                    );
                    return;
                }
            };

            diesel::update(
                assigned_subaddresses::table.filter(
                    assigned_subaddresses::public_address_b58
                        .eq(&assigned_subaddress.public_address_b58),
                ),
            )
            .set((assigned_subaddresses::spend_public_key.eq(&spend_public_key_bytes),))
            .execute(conn)
            .expect("failed updating assigned subaddress");

            global_log::debug!("Assigned subaddress proto conversion done");
        }
    }
}

/// Escape a string for consumption by SQLite.
/// This function doubles all single quote characters within the string, then
/// wraps the string in single quotes on the front and back.
fn sql_escape_string(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}

/// Create an immediate SQLite transaction with retry.
/// Note: This function does not support nested transactions.
pub fn exclusive_transaction<T, E, F>(conn: Conn, f: F) -> Result<T, E>
where
    F: Clone + FnOnce(&mut SqliteConnection) -> Result<T, E>,
    E: From<diesel::result::Error>,
{
    for i in 0..NUM_RETRIES {
        let r = conn.exclusive_transaction::<T, E, F>(f.clone());
        if r.is_ok() || i == (NUM_RETRIES - 1) {
            return r;
        }
        sleep(Duration::from_millis((BASE_DELAY_MS * 2_u32.pow(i)) as u64));
    }
    panic!("Should never reach this point.");
}

const BASE_DELAY_MS: u32 = 10;
const NUM_RETRIES: u32 = 5;
