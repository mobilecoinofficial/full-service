use clap::Parser;
use mc_common::logger::{log, Logger};
use mc_ledger_db::LedgerDB;
use mc_util_uri::{Uri, UriScheme};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

use crate::{db::Conn, error::T3SyncError, WalletDb};

#[derive(Debug, Hash, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct T3Scheme {}
impl UriScheme for T3Scheme {
    /// The part before the '://' of a URL.
    const SCHEME_SECURE: &'static str = "t3";
    const SCHEME_INSECURE: &'static str = "insecure-t3";

    /// Default port numbers
    const DEFAULT_SECURE_PORT: u16 = 443;
    const DEFAULT_INSECURE_PORT: u16 = 3223;
}

pub type T3Uri = Uri<T3Scheme>;

#[derive(Clone, Debug, Parser)]
pub struct T3SyncConfig {
    #[clap(long, env = "T3_URI")]
    pub t3_uri: T3Uri,
    #[clap(long, env = "T3_API_KEY")]
    pub t3_api_key: String,
}

// How many txos to sync per round
const TXO_CHUNK_SIZE: usize = 5;
// How long to wait in milliseconds between sync rounds
const T3_SYNC_INTERVAL: u64 = 1000;

/// T3 Sync thread - holds objects needed to cleanly terminate the t3 sync
/// thread.
pub struct T3SyncThread {
    /// The configuration for the t3 sync thread.
    config: T3SyncConfig,

    /// The main sync thread handle.
    join_handle: Option<thread::JoinHandle<()>>,

    /// Stop trigger, used to signal the thread to terminate.
    stop_requested: Arc<AtomicBool>,
}

impl T3SyncThread {
    pub fn start(
        config: T3SyncConfig,
        ledger_db: LedgerDB,
        wallet_db: WalletDb,
        logger: Logger,
    ) -> Self {
        let stop_requested = Arc::new(AtomicBool::new(false));
        let thread_stop_requested = stop_requested.clone();

        let join_handle = Some(
            thread::Builder::new()
                .name("t3_sync".to_string())
                .spawn(move || {
                    log::debug!(logger, "T3 Sync thread started.");

                    let conn = &mut wallet_db
                        .get_pooled_conn()
                        .expect("failed getting wallet db connection");

                    loop {
                        if thread_stop_requested.load(Ordering::SeqCst) {
                            log::debug!(logger, "T3SyncThread stop requested.");
                            break;
                        }

                        match sync_txos(&ledger_db, conn, &logger) {
                            Ok(()) => (),
                            Err(e) => log::error!(&logger, "Error during t3 sync:\n{:?}", e),
                        }

                        // This sleep is to allow other API calls that need access to the database a
                        // chance to execute, because the t3 sync process requires a write lock on
                        // the database.
                        thread::sleep(std::time::Duration::from_millis(T3_SYNC_INTERVAL));
                    }
                    log::debug!(logger, "T3SyncThread stopped.");
                })
                .expect("failed starting t3 sync thread"),
        );

        Self {
            config,
            join_handle,
            stop_requested,
        }
    }

    pub fn stop(&mut self) {
        self.stop_requested.store(true, Ordering::SeqCst);
        if let Some(join_handle) = self.join_handle.take() {
            join_handle.join().expect("T3SyncThread join failed");
        }
    }
}

impl Drop for T3SyncThread {
    fn drop(&mut self) {
        self.stop();
    }
}

pub fn sync_txos(ledger_db: &LedgerDB, conn: Conn, logger: &Logger) -> Result<(), T3SyncError> {
    // get all txos from the database that haven't synced yet to t3 and that have an
    // authenticated sender memo
    log::debug!(logger, "Syncing txos to t3");
    Ok(())
}
