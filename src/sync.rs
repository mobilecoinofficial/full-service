// Copyright (c) 2018-2020 MobileCoin Inc.

//! Manages ledger block scanning for wallet accounts..
//!
//! Note: Copied and reworked from mobilecoin/mobilecoind/src/sync.rs. Future work is to figure out
//!       how to better share this code.
//!
//! The sync code creates a pool of worker threads, and a main thread to hand off tasks to the
//! worker threads over a crossbeam channel. Each task is a request to sync block data for a given
//! account id. Each task is limited to a pre-defined amount of blocks - this is useful when the
//! amount of accounts exceeds the amount of working threads as it ensures accounts are processed
//! concurrently.
//!
//! The main thread periodically queries the database for all currently known account ids, and
//! submits new jobs into the queue for each account not currently queued. In order to prevent
//! duplicate queueing, the code also keeps track of the list of already-queued account ids inside
//! a hashset that is shared with the worker threads. When a worker thread is finished with a given
//! account id, it removes it from the hashset, which in turns allows the main thread to queue it
//! again once the polling interval is exceeded. Since the worker thread processes blocks in
//! chunks, it is possible that not all available blocks gets processed at once. When that happens,
//! instead of removing the account id from the hashset, it would be placed back into the queue to
//! be picked up by the next available worker thread.

use crate::{
    db::{AccountID, WalletDb},
    error::{SyncError, WalletDbError},
    models::Account,
};
use mc_account_keys::AccountKey;
use mc_common::{
    logger::{log, Logger},
    HashSet,
};
use mc_crypto_keys::RistrettoPublic;
use mc_ledger_db::{Ledger, LedgerDB};
use mc_transaction_core::{
    get_tx_out_shared_secret,
    onetime_keys::{recover_onetime_private_key, recover_public_subaddress_spend_key},
    ring_signature::KeyImage,
    tx::TxOut,
};
use std::{
    convert::TryFrom,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
};

///  The maximal number of blocks a worker thread would process at once.
const MAX_BLOCKS_PROCESSING_CHUNK_SIZE: usize = 5;

/// The MonitorId corresponds to the Account's primary key: account_key_hex.
pub type MonitorId = String;

/// Message type our crossbeam channel uses to communicate with the worker thread pull.
enum SyncMsg {
    SyncMonitor(MonitorId),
    Stop,
}

/// Possible return values for the `sync_monitor` function.
#[derive(Debug, Eq, PartialEq)]
enum SyncMonitorOk {
    /// No more blocks are currently available for processing.
    NoMoreBlocks,

    /// More blocks might be available.
    MoreBlocksPotentiallyAvailable,
}

/// Sync thread - holds objects needed to cleanly terminate the sync thread.
pub struct SyncThread {
    /// The main sync thread handle.
    join_handle: Option<thread::JoinHandle<()>>,

    /// Stop trigger, used to signal the thread to reminate.
    stop_requested: Arc<AtomicBool>,
}

impl SyncThread {
    pub fn start(
        ledger_db: LedgerDB,
        wallet_db: WalletDb,
        num_workers: Option<usize>,
        logger: Logger,
    ) -> Self {
        // Queue for sending jobs to our worker threads.
        let (sender, receiver) = crossbeam_channel::unbounded::<SyncMsg>();

        // A hashset to keep track of which MonitorIds were already sent to the queue,
        // preventing them from being sent again until they are processed.
        let queued_monitor_ids = Arc::new(Mutex::new(HashSet::<MonitorId>::default()));

        // Create worker threads.
        let mut worker_join_handles = Vec::new();

        for idx in 0..num_workers.unwrap_or_else(num_cpus::get) {
            let thread_ledger_db = ledger_db.clone();
            let thread_wallet_db = wallet_db.clone();
            let thread_sender = sender.clone();
            let thread_receiver = receiver.clone();
            let thread_queued_monitor_ids = queued_monitor_ids.clone();
            let thread_logger = logger.clone();
            let join_handle = thread::Builder::new()
                .name(format!("sync_worker_{}", idx))
                .spawn(move || {
                    sync_thread_entry_point(
                        thread_ledger_db,
                        thread_wallet_db,
                        thread_sender,
                        thread_receiver,
                        thread_queued_monitor_ids,
                        thread_logger,
                    );
                })
                .expect("failed starting sync worker thread");

            worker_join_handles.push(join_handle);
        }

        // Start the main sync thread.
        // This thread constantly monitors the list of monitor ids we are aware of,
        // and adds new one into our cyclic queue.
        let stop_requested = Arc::new(AtomicBool::new(false));
        let thread_stop_requested = stop_requested.clone();

        let join_handle = Some(
            thread::Builder::new()
                .name("sync".to_string())
                .spawn(move || {
                    log::debug!(logger, "Syncthread started.");

                    loop {
                        if thread_stop_requested.load(Ordering::SeqCst) {
                            log::debug!(logger, "SyncThread stop requested.");
                            break;
                        }

                        // Get the current number of blocks in ledger.
                        let num_blocks = ledger_db
                            .num_blocks()
                            .expect("failed getting number of blocks");

                        // A flag to track whether we sent a message to our work queue.
                        // If we sent a message, that means new blocks have arrived and we can skip sleeping.
                        // If no new blocks arrived, and we haven't had to sync any monitors, we can sleep for
                        // a bit so that we do not use 100% cpu.
                        let mut message_sent = false;

                        // Go over our list of accounts and see which one needs to process these blocks.
                        for account in wallet_db
                            .list_accounts()
                            .expect("failed getting accounts from WalletDb")
                        {
                            // If there are no new blocks for this account, don't do anything.
                            if account.next_block >= num_blocks as i64 {
                                println!("\x1b[1;31m SYNC: No new blocks for account\x1b[0m");
                                continue;
                            }

                            let mut queued_monitor_ids =
                                queued_monitor_ids.lock().expect("mutex poisoned");
                            if !queued_monitor_ids.insert(account.account_id_hex.clone()) {
                                // Already queued, no need to add again to queue at this point.
                                log::trace!(
                                    logger,
                                    "{}: skipping, already queued",
                                    account.account_id_hex
                                );
                                continue;
                            }

                            // This account has blocks to process, put it in the queue.
                            log::debug!(
                                logger,
                                "sync thread noticed monitor {} needs syncing",
                                account.account_id_hex,
                            );
                            sender
                                .send(SyncMsg::SyncMonitor(account.account_id_hex))
                                .expect("failed sending to queue");
                            message_sent = true;
                        }

                        // If we saw no activity, sleep for a bit.
                        if !message_sent {
                            thread::sleep(std::time::Duration::from_secs(1));
                        }
                    }

                    log::trace!(
                        logger,
                        "SyncThread attempting to stop all worker threads..."
                    );
                    for _ in 0..worker_join_handles.len() {
                        sender
                            .send(SyncMsg::Stop)
                            .expect("failed sending stop message");
                    }

                    let num_workers = worker_join_handles.len();
                    for (i, join_handle) in worker_join_handles.into_iter().enumerate() {
                        log::trace!(logger, "Joining worker {}/{}", i + 1, num_workers);
                        join_handle.join().expect("Failed joining worker thread");
                        log::debug!(
                            logger,
                            "SyncThread worker {}/{} stopped",
                            i + 1,
                            num_workers
                        );
                    }

                    log::debug!(logger, "SyncThread stopped.");
                })
                .expect("failed starting main sync thread"),
        );

        Self {
            join_handle,
            stop_requested,
        }
    }

    pub fn stop(&mut self) {
        self.stop_requested.store(true, Ordering::SeqCst);
        if let Some(join_handle) = self.join_handle.take() {
            join_handle.join().expect("SyncThread join failed");
        }
    }
}

impl Drop for SyncThread {
    fn drop(&mut self) {
        self.stop();
    }
}
/// The entry point of a sync worker thread that processes queue messages.
fn sync_thread_entry_point(
    ledger_db: LedgerDB,
    wallet_db: WalletDb,
    sender: crossbeam_channel::Sender<SyncMsg>,
    receiver: crossbeam_channel::Receiver<SyncMsg>,
    queued_monitor_ids: Arc<Mutex<HashSet<MonitorId>>>,
    logger: Logger,
) {
    for msg in receiver.iter() {
        match msg {
            SyncMsg::SyncMonitor(monitor_id) => {
                match sync_monitor(&ledger_db, &wallet_db, &monitor_id, &logger) {
                    // Success - No more blocks are currently available.
                    Ok(SyncMonitorOk::NoMoreBlocks) => {
                        // Remove the monitor id from the list of queued ones so that the main thread could
                        // queue it again if necessary.
                        log::trace!(logger, "{}: sync_monitor returned NoMoreBlocks", monitor_id);

                        let mut queued_monitor_ids =
                            queued_monitor_ids.lock().expect("mutex poisoned");
                        queued_monitor_ids.remove(&monitor_id);
                    }

                    // Success - more blocks might be available.
                    Ok(SyncMonitorOk::MoreBlocksPotentiallyAvailable) => {
                        // Put the monitor id back in the queue for further processing.
                        log::trace!(
                            logger,
                            "{}: sync_monitor returned MoreBlocksPotentiallyAvailable",
                            monitor_id,
                        );

                        sender
                            .send(SyncMsg::SyncMonitor(monitor_id))
                            .expect("failed sending to channel");
                    }

                    // Errors that are acceptable - nothing to do.
                    Err(SyncError::AccountNotFound) => {}

                    // Other errors - log.
                    Err(err) => {
                        log::error!(logger, "error syncing monitor {}: {:?}", monitor_id, err);
                    }
                };
            }

            SyncMsg::Stop => {
                break;
            }
        }
    }
}

/// Sync a single monitor.
fn sync_monitor(
    ledger_db: &LedgerDB,
    wallet_db: &WalletDb,
    monitor_id: &MonitorId,
    logger: &Logger,
) -> Result<SyncMonitorOk, SyncError> {
    for _ in 0..MAX_BLOCKS_PROCESSING_CHUNK_SIZE {
        // Get the account data. If it is no longer available, the monitor has been removed and we
        // can simply return. FIXME - verify this works as intended with new data model
        println!("\x1b[1;33m \tSYNC: now getting account \x1b[0m");
        let account = wallet_db.get_account(monitor_id)?;
        let block_contents = match ledger_db.get_block_contents(account.next_block as u64) {
            Ok(block_contents) => block_contents,
            Err(mc_ledger_db::Error::NotFound) => {
                return Ok(SyncMonitorOk::NoMoreBlocks);
            }
            Err(err) => {
                return Err(err.into());
            }
        };

        log::trace!(
            logger,
            "processing {} outputs and {} key images from block {} for account {}",
            block_contents.outputs.len(),
            block_contents.key_images.len(),
            account.next_block,
            monitor_id,
        );

        // Match tx outs into UTXOs.
        println!("\x1b[1;34m \tSYNC: now processing utxos \x1b[0m");
        process_txos(
            &wallet_db,
            &block_contents.outputs,
            &account,
            account.next_block,
            logger,
        )?;

        println!("\x1b[1;35m \tSYNC: now updating spent and incrementing block \x1b[0m");
        // Note: Doing this here means we are updating key images multiple times, once per account.
        //       We do actually want to do it this way, because each account may need to process
        //       the same block at a different time, depending on when we add it to the DB.
        wallet_db.update_spent_and_increment_next_block(
            &account.account_id_hex,
            account.next_block,
            block_contents.key_images,
        )?;
    }

    Ok(SyncMonitorOk::MoreBlocksPotentiallyAvailable)
}

/// Helper function for matching a list of TxOuts to a given monitor.
fn process_txos(
    wallet_db: &WalletDb,
    outputs: &[TxOut],
    account: &Account,
    received_block_index: i64,
    logger: &Logger,
) -> Result<(), SyncError> {
    let account_key: AccountKey = mc_util_serial::decode(&account.encrypted_account_key)?;
    let view_key = account_key.view_key();

    for tx_out in outputs {
        // Calculate the subaddress spend public key for tx_out.
        let tx_out_target_key = RistrettoPublic::try_from(&tx_out.target_key)?;
        let tx_public_key = RistrettoPublic::try_from(&tx_out.public_key)?;

        let subaddress_spk: RistrettoPublic = recover_public_subaddress_spend_key(
            &view_key.view_private_key,
            &tx_out_target_key,
            &tx_public_key,
        );

        println!("\x1b[1;31m \tSYNC: getting subaddress by spend public key \x1b[0m");
        // See if it matches any of our assigned subaddresses.
        let (subaddress_index, account_id_hex) =
            match wallet_db.get_subaddress_index_by_subaddress_spend_public_key(&subaddress_spk) {
                Ok((index, account_id)) => {
                    log::trace!(
                        logger,
                        "matched subaddress index {} for monitor_id {}",
                        index,
                        account_id,
                    );
                    (index, account_id)
                }
                Err(WalletDbError::NotFound(_)) => continue,
                Err(err) => {
                    return Err(err.into());
                }
            };

        // Sanity - we should only get a match for our own account ID.
        assert_eq!(AccountID::from(&account_key).to_string(), account_id_hex);

        let shared_secret =
            get_tx_out_shared_secret(account_key.view_private_key(), &tx_public_key);

        let (value, _blinding) = tx_out
            .amount
            .get_value(&shared_secret)
            .expect("Malformed amount"); // TODO

        let onetime_private_key = recover_onetime_private_key(
            &tx_public_key,
            account_key.view_private_key(),
            &account_key.subaddress_spend_private(subaddress_index as u64),
        );

        let key_image = KeyImage::from(&onetime_private_key);

        println!("\x1b[1;32m \tSYNC: now creating a received TXO \x1b[0m");
        // Insert received txo
        wallet_db.create_received_txo(
            tx_out.clone(),
            subaddress_index as u64, // FIXME: precision...
            key_image,
            value,
            received_block_index,
            &account_id_hex,
        )?;
    }

    Ok(())
}

// FIXME: Add tests
