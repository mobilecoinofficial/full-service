use crate::{
    db::{
        models::{AuthenticatedSenderMemo, Txo},
        txo::{TxoMemo, TxoModel},
        Conn,
    },
    error::T3SyncError,
    WalletDb,
};
use clap::Parser;
use mc_account_keys::ShortAddressHash;
use mc_common::logger::{log, Logger};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};
use t3_api::{ReportedDirection, T3Uri, TransparentTransaction};
use t3_connection::T3Connection;

#[derive(Clone, Debug, Parser, Default)]
pub struct T3Config {
    #[clap(long, env = "T3_URI", requires = "t3_api_key")]
    pub t3_uri: Option<T3Uri>,
    #[clap(long, env = "T3_API_KEY", requires = "t3_uri")]
    pub t3_api_key: Option<String>,
}

// How many txos to sync per round
// TODO - Discuss in PR if this is a reasonable value
const TXO_CHUNK_SIZE: usize = 5;
// How long to wait in milliseconds between sync rounds
// TODO - discuss in PR if this is a reasonable value
const T3_SYNC_INTERVAL: Duration = Duration::from_millis(1000);

/// T3 Sync thread - holds objects needed to cleanly terminate the t3 sync
/// thread.
pub struct T3SyncThread {
    /// The main sync thread handle.
    join_handle: Option<thread::JoinHandle<()>>,

    /// Stop trigger, used to signal the thread to terminate.
    stop_requested: Arc<AtomicBool>,
}

impl T3SyncThread {
    pub fn start(t3_uri: T3Uri, t3_api_key: String, wallet_db: WalletDb, logger: Logger) -> Self {
        let stop_requested = Arc::new(AtomicBool::new(false));
        let thread_stop_requested = stop_requested.clone();

        let t3_connection = T3Connection::new(&t3_uri, t3_api_key, logger.clone());

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

                        match sync_txos(conn, &t3_connection, &logger) {
                            Ok(()) => (),
                            Err(e) => log::error!(&logger, "Error during t3 sync:\n{:?}", e),
                        }

                        // This sleep is to allow other API calls that need access to the database a
                        // chance to execute, because the t3 sync process requires a write lock on
                        // the database.
                        thread::sleep(T3_SYNC_INTERVAL);
                    }
                    log::debug!(logger, "T3SyncThread stopped.");
                })
                .expect("failed starting t3 sync thread"),
        );

        Self {
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

pub fn sync_txos(
    conn: Conn,
    t3_connection: &T3Connection,
    logger: &Logger,
) -> Result<(), T3SyncError> {
    // get all txos from the database that haven't synced yet to t3 and that have an
    // authenticated sender memo
    log::debug!(logger, "Syncing txos to t3");

    let txos = Txo::get_txos_that_need_to_be_synced_to_t3(Some(TXO_CHUNK_SIZE), conn)?;

    for txo in txos {
        let txo_memo = txo.memo(conn)?;
        let memo = match txo_memo {
            TxoMemo::AuthenticatedSender(memo) => memo,
            _ => return Err(T3SyncError::TxoMemoIsNotAuthenticatedSender(txo.id)),
        };

        let recipient_short_address_hash = txo
            .recipient_public_address(conn)?
            .map(|address| (&address).into());

        sync_txo(&txo, &memo, recipient_short_address_hash, t3_connection)?;
        txo.update_is_synced_to_t3(true, conn)?;
    }

    Ok(())
}

fn sync_txo(
    txo: &Txo,
    memo: &AuthenticatedSenderMemo,
    recipient_short_address_hash: Option<ShortAddressHash>,
    t3_connection: &T3Connection,
) -> Result<(), T3SyncError> {
    let mut transparent_transaction = TransparentTransaction::new();

    let sender_address_hash_bytes = hex::decode(&memo.sender_address_hash)?;
    transparent_transaction.set_sender_address_hash(sender_address_hash_bytes);

    // now we need to set the receiver address hash, which will require a database
    // lookup, but only if it exists
    if let Some(recipient_short_address_hash) = recipient_short_address_hash {
        transparent_transaction
            .set_recipient_address_hash(recipient_short_address_hash.as_ref().to_vec());
    }

    transparent_transaction.set_token_id(txo.token_id as u64);
    transparent_transaction.set_amount(txo.value as u64);

    let reported_direction = match txo.account_id {
        Some(_) => ReportedDirection::REPORTED_DIRECTION_RECEIVE,
        None => ReportedDirection::REPORTED_DIRECTION_SEND,
    };
    transparent_transaction.set_reported_direction(reported_direction);

    let public_key_bytes = txo.public_key()?.as_bytes().to_vec();
    transparent_transaction.set_public_key_hex(hex::encode(public_key_bytes));

    let _ = t3_connection.create_transaction(transparent_transaction)?;

    Ok(())
}
