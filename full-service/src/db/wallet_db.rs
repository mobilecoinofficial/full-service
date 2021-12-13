use crate::db::WalletDbError;
use diesel::{
    connection::SimpleConnection,
    prelude::*,
    r2d2::{ConnectionManager, Pool, PooledConnection},
    sql_types,
};
use mc_common::logger::{global_log, Logger};
use std::{env, time::Duration};

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
            if let Some(d) = self.busy_timeout {
                conn.batch_execute(&format!("PRAGMA busy_timeout = {};", d.as_millis()))?;
            }

            Ok(())
        })()
        .map_err(diesel::r2d2::Error::QueryError)
    }
}

#[derive(Clone)]
pub struct WalletDb {
    pool: Pool<ConnectionManager<SqliteConnection>>,
    logger: Logger,
}

impl WalletDb {
    pub fn new(pool: Pool<ConnectionManager<SqliteConnection>>, logger: Logger) -> Self {
        Self { pool, logger }
    }

    pub fn new_from_url(
        database_url: &str,
        db_connections: u32,
        logger: Logger,
    ) -> Result<Self, WalletDbError> {
        let manager = ConnectionManager::<SqliteConnection>::new(database_url);
        let pool = Pool::builder()
            .max_size(db_connections)
            .connection_customizer(Box::new(ConnectionOptions {
                enable_wal: true,
                enable_foreign_keys: false,
                busy_timeout: Some(Duration::from_secs(30)),
            }))
            .test_on_check_out(true)
            .build(manager)?;
        Ok(Self::new(pool, logger))
    }

    pub fn get_conn(
        &self,
    ) -> Result<PooledConnection<ConnectionManager<SqliteConnection>>, WalletDbError> {
        Ok(self.pool.get()?)
    }

    pub fn set_db_encryption_key_from_env(conn: &SqliteConnection) {
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

    pub fn try_change_db_encryption_key_from_env(conn: &SqliteConnection) {
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

    pub fn check_database_connectivity(conn: &SqliteConnection) -> bool {
        conn.batch_execute("SELECT count(*) FROM sqlite_master;")
            .is_ok()
    }

    pub fn validate_foreign_keys(conn: &SqliteConnection) {
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
}

/// Escape a string for consumption by SQLite.
/// This function doubles all single quote characters within the string, then
/// wraps the string in single quotes on the front and back.
fn sql_escape_string(s: &str) -> String {
    format!("'{}'", s.replace("'", "''"))
}
