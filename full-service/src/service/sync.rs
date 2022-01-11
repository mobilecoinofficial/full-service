// Copyright (c) 2018-2020 MobileCoin Inc.

//! Manages ledger block scanning for wallet accounts..
//!
//! Note: Copied and reworked from mobilecoin/mobilecoind/src/sync.rs. Future
//! work is to figure out       how to better share this code.
//!
//! The sync code creates a pool of worker threads, and a main thread to hand
//! off tasks to the worker threads over a crossbeam channel. Each task is a
//! request to sync block data for a given account id. Each task is limited to a
//! pre-defined amount of blocks - this is useful when the amount of accounts
//! exceeds the amount of working threads as it ensures accounts are processed
//! concurrently.
//!
//! The main thread periodically queries the database for all currently known
//! account ids, and submits new jobs into the queue for each account not
//! currently queued. In order to prevent duplicate queueing, the code also
//! keeps track of the list of already-queued account ids inside a hashset that
//! is shared with the worker threads. When a worker thread is finished with a
//! given account id, it removes it from the hashset, which in turns allows the
//! main thread to queue it again once the polling interval is exceeded. Since
//! the worker thread processes blocks in chunks, it is possible that not all
//! available blocks gets processed at once. When that happens, instead of
//! removing the account id from the hashset, it would be placed back into the
//! queue to be picked up by the next available worker thread.

use crate::{
    db::{
        account::{AccountID, AccountModel},
        assigned_subaddress::AssignedSubaddressModel,
        models::{Account, AssignedSubaddress, TransactionLog, Txo},
        transaction_log::TransactionLogModel,
        txo::TxoModel,
        WalletDb, WalletDbError,
    },
    error::SyncError,
};
use mc_account_keys::AccountKey;
use mc_common::{
    logger::{log, Logger},
    HashMap, HashSet,
};
use mc_crypto_keys::RistrettoPublic;
use mc_ledger_db::{Ledger, LedgerDB};
use mc_transaction_core::{
    get_tx_out_shared_secret,
    onetime_keys::{recover_onetime_private_key, recover_public_subaddress_spend_key},
    ring_signature::KeyImage,
    tx::TxOut,
    AmountError,
};

use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, PooledConnection},
};
use std::{
    convert::TryFrom,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

///  The maximal number of blocks a worker thread would process at once.
const MAX_BLOCKS_PROCESSING_CHUNK_SIZE: usize = 5;

/// The AccountId corresponds to the Account's primary key: account_key_hex.
pub type AccountId = String;

/// Message type our crossbeam channel uses to communicate with the worker
/// thread pull.
enum SyncMsg {
    SyncAccount(AccountId),
    Stop,
}

/// Possible return values for the `sync_account` function.
#[derive(Debug, Eq, PartialEq)]
pub enum SyncAccountOk {
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

        // A hashset to keep track of which AccountIds were already sent to the queue,
        // preventing them from being sent again until they are processed.
        let queued_account_ids = Arc::new(Mutex::new(HashSet::<AccountId>::default()));

        // Create worker threads.
        let mut worker_join_handles = Vec::new();

        for idx in 0..num_workers.unwrap_or_else(num_cpus::get) {
            let thread_ledger_db = ledger_db.clone();
            let thread_wallet_db = wallet_db.clone();
            let thread_sender = sender.clone();
            let thread_receiver = receiver.clone();
            let thread_queued_account_ids = queued_account_ids.clone();
            let thread_logger = logger.clone();
            let join_handle = thread::Builder::new()
                .name(format!("sync_worker_{}", idx))
                .spawn(move || {
                    sync_thread_entry_point(
                        thread_ledger_db,
                        thread_wallet_db,
                        thread_sender,
                        thread_receiver,
                        thread_queued_account_ids,
                        thread_logger,
                    );
                })
                .expect("failed starting sync worker thread");

            worker_join_handles.push(join_handle);
        }

        // Start the main sync thread.
        // This thread constantly accounts the list of account ids we are aware of,
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
                        // If we sent a message, that means new blocks have arrived and we can skip
                        // sleeping. If no new blocks arrived, and we
                        // haven't had to sync any accounts, we can sleep for
                        // a bit so that we do not use 100% cpu.
                        let mut message_sent = false;

                        // Go over our list of accounts and see which one needs to process these
                        // blocks.
                        let accounts = {
                            let conn = &wallet_db
                                .get_conn()
                                .expect("Could not get connection to DB");
                            conn.transaction::<Vec<Account>, WalletDbError, _>(|| {
                                Ok(Account::list_all(conn)
                                    .expect("Failed getting accounts from database"))
                            })
                            .expect("Failed executing database transaction")
                        };
                        for account in accounts {
                            // If there are no new blocks for this account, don't do anything.
                            if account.next_block_index >= num_blocks as i64 {
                                continue;
                            }

                            let mut queued_account_ids =
                                queued_account_ids.lock().expect("mutex poisoned");
                            if !queued_account_ids.insert(account.account_id_hex.clone()) {
                                // Already queued, no need to add again to queue at this point.
                                log::trace!(
                                    logger,
                                    "{}: skipping, already queued {} with next_block_index {} at num_blocks {}",
                                    account.account_id_hex,
                                    account.name,
                                    account.next_block_index,
                                    num_blocks
                                );
                                // If sync failed on this account due to DB lock previously, we will
                                // need to make sure it is still in the queue.
                                if !sender.is_empty() {
                                    continue;
                                }
                            }

                            // This account has blocks to process, put it in the queue.
                            log::debug!(
                                logger,
                                "sync thread noticed account {} {} with next_block_index {} needs syncing at num_blocks {}",
                                account.account_id_hex,
                                account.name,
                                account.next_block_index,
                                num_blocks
                            );
                            sender
                                .send(SyncMsg::SyncAccount(account.account_id_hex))
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
    queued_account_ids: Arc<Mutex<HashSet<AccountId>>>,
    logger: Logger,
) {
    for msg in receiver.iter() {
        match msg {
            SyncMsg::SyncAccount(account_id) => {
                match sync_account(&ledger_db, &wallet_db, &account_id, &logger) {
                    // Success - No more blocks are currently available.
                    Ok(SyncAccountOk::NoMoreBlocks) => {
                        // Remove the account id from the list of queued ones so that the main
                        // thread could queue it again if necessary.
                        log::trace!(logger, "{}: sync_account returned NoMoreBlocks", account_id);

                        let mut queued_account_ids =
                            queued_account_ids.lock().expect("mutex poisoned");
                        queued_account_ids.remove(&account_id);
                    }

                    // Success - more blocks might be available.
                    Ok(SyncAccountOk::MoreBlocksPotentiallyAvailable) => {
                        // Put the account id back in the queue for further processing.
                        log::trace!(
                            logger,
                            "{}: sync_account returned MoreBlocksPotentiallyAvailable",
                            account_id,
                        );

                        sender
                            .send(SyncMsg::SyncAccount(account_id))
                            .expect("failed sending to channel");
                    }

                    // Errors that are acceptable - nothing to do.
                    Err(SyncError::AccountNotFound) => {}

                    // Database is locked means there was some write contention, which is expected
                    // when using SQLite3 with concurrency. Fail gracefully and retry on next loop.
                    Err(SyncError::Database(WalletDbError::Diesel(
                        diesel::result::Error::DatabaseError(kind, info),
                    ))) => {
                        match info.message() {
                            "database is locked" => {
                                log::trace!(logger, "Database locked. Will retry")
                            }
                            _ => log::error!(
                                logger,
                                "Unexpected database error {:?} {:?} {:?} {:?} {:?} {:?}",
                                kind,
                                info,
                                info.details(),
                                info.column_name(),
                                info.table_name(),
                                info.hint(),
                            ),
                        };
                        // Sleep for half a second to let the database finish what it's up to
                        std::thread::sleep(Duration::from_millis(500));
                    }

                    // Other errors - log.
                    Err(err) => {
                        log::error!(logger, "error syncing account {}: {:?}", account_id, err);
                    }
                };
            }

            SyncMsg::Stop => {
                break;
            }
        }
    }
}

/// Sync a single account.
pub fn sync_account(
    ledger_db: &LedgerDB,
    wallet_db: &WalletDb,
    account_id: &str,
    logger: &Logger,
) -> Result<SyncAccountOk, SyncError> {
    for _ in 0..MAX_BLOCKS_PROCESSING_CHUNK_SIZE {
        let conn = wallet_db.get_conn()?;
        let sync_status = conn.transaction::<SyncAccountOk, SyncError, _>(|| {
            // Get the account data. If it is no longer available, the account has been
            // removed and we can simply return.
            let account = Account::get(&AccountID(account_id.to_string()), &conn)?;
            let block_contents = match ledger_db.get_block_contents(account.next_block_index as u64)
            {
                Ok(block_contents) => block_contents,
                Err(mc_ledger_db::Error::NotFound) => {
                    return Ok(SyncAccountOk::NoMoreBlocks);
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
                account.next_block_index,
                account_id,
            );

            // Match tx outs into UTXOs.
            let output_txo_ids = process_txos(
                &conn,
                &block_contents.outputs,
                &account,
                account.next_block_index,
                logger,
            )?;

            // Note: Doing this here means we are updating key images multiple times, once
            // per account. We do actually want to do it this way, because each account may
            // need to process the same block at a different time, depending on when we add
            // it to the DB.
            account.update_spent_and_increment_next_block(
                account.next_block_index,
                block_contents.key_images,
                &conn,
            )?;

            // Add a transaction for the received TXOs
            TransactionLog::log_received(
                &output_txo_ids,
                &account,
                account.next_block_index as u64,
                &conn,
            )?;

            Ok(SyncAccountOk::MoreBlocksPotentiallyAvailable)
        })?;
        // Early out of the loop if we hit NoMoreBlocks
        if let SyncAccountOk::NoMoreBlocks = sync_status {
            return Ok(SyncAccountOk::NoMoreBlocks);
        }
    }
    Ok(SyncAccountOk::MoreBlocksPotentiallyAvailable)
}

/// Helper function for matching a list of TxOuts to a given account.
pub fn process_txos(
    conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    outputs: &[TxOut],
    account: &Account,
    received_block_index: i64,
    logger: &Logger,
) -> Result<HashMap<i64, Vec<String>>, SyncError> {
    let account_key: AccountKey = mc_util_serial::decode(&account.account_key)?;
    let view_key = account_key.view_key();
    let account_id_hex = AccountID::from(&account_key).to_string();

    let mut output_txo_ids: HashMap<i64, Vec<String>> = HashMap::default();

    for tx_out in outputs {
        // Calculate the subaddress spend public key for tx_out.
        let tx_out_target_key = RistrettoPublic::try_from(&tx_out.target_key)?;
        let tx_public_key = RistrettoPublic::try_from(&tx_out.public_key)?;

        let subaddress_spk: RistrettoPublic = recover_public_subaddress_spend_key(
            &view_key.view_private_key,
            &tx_out_target_key,
            &tx_public_key,
        );

        // See if it matches any of our assigned subaddresses.
        let subaddress_index =
            match AssignedSubaddress::find_by_subaddress_spend_public_key(&subaddress_spk, conn) {
                Ok((index, account_id)) => {
                    log::trace!(
                        logger,
                        "matched subaddress index {} for account_id {}",
                        index,
                        account_id,
                    );
                    // Sanity - we should only get a match for our own account ID.
                    assert_eq!(account_id, account_id_hex);
                    Some(index)
                }
                Err(WalletDbError::AssignedSubaddressNotFound(_)) => {
                    log::trace!(
                        logger,
                        "Not tracking this subaddress spend public key for account {}",
                        account_id_hex
                    );
                    None
                }
                Err(err) => {
                    return Err(err.into());
                }
            };

        let shared_secret =
            get_tx_out_shared_secret(account_key.view_private_key(), &tx_public_key);

        let value = match tx_out.amount.get_value(&shared_secret) {
            Ok((v, _blinding)) => v,
            Err(AmountError::InconsistentCommitment) => {
                // Assume this is not a transaction that belongs to us. We go this far because
                // we are trying to match txos even if we did not preemptively store a
                // subaddress spk.
                continue;
            }
        };

        let key_image = subaddress_index.map(|subaddress_i| {
            let onetime_private_key = recover_onetime_private_key(
                &tx_public_key,
                account_key.view_private_key(),
                &account_key.subaddress_spend_private(subaddress_i as u64),
            );
            KeyImage::from(&onetime_private_key)
        });

        // Insert received txo
        let txo_id = Txo::create_received(
            tx_out.clone(),
            subaddress_index,
            key_image,
            value,
            received_block_index,
            &account_id_hex,
            conn,
        )?;

        // If we couldn't find an assigned subaddress for this value, store for -1
        let subaddress_key: i64 = subaddress_index.unwrap_or(-1) as i64;
        if output_txo_ids.get(&(subaddress_key)).is_none() {
            output_txo_ids.insert(subaddress_key, Vec::new());
        }

        output_txo_ids
            .get_mut(&(subaddress_key))
            .unwrap() // We know the key exists because we inserted above
            .push(txo_id);
    }

    Ok(output_txo_ids)
}

// FIXME: test select received txo by value
// FIXME: test syncing after removing account

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        service::{account::AccountService, balance::BalanceService},
        test_utils::{add_block_to_ledger_db, get_test_ledger, setup_wallet_service, MOB},
    };
    use mc_account_keys::{AccountKey, RootEntropy, RootIdentity};
    use mc_common::logger::{test_with_logger, Logger};
    use mc_util_from_random::FromRandom;
    use rand::{rngs::StdRng, SeedableRng};

    #[test_with_logger]
    fn test_process_txo_bigint_in_origin(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let entropy = RootEntropy::from_random(&mut rng);
        let account_key = AccountKey::from(&RootIdentity::from(&entropy));

        let mut ledger_db = get_test_ledger(0, &vec![], 0, &mut rng);

        let origin_block_amount: u128 = 250_000_000 * MOB as u128;
        let origin_block_txo_amount = origin_block_amount / 16;
        let o = account_key.subaddress(0);
        let _new_block_index = add_block_to_ledger_db(
            &mut ledger_db,
            &[
                o.clone(),
                o.clone(),
                o.clone(),
                o.clone(),
                o.clone(),
                o.clone(),
                o.clone(),
                o.clone(),
                o.clone(),
                o.clone(),
                o.clone(),
                o.clone(),
                o.clone(),
                o.clone(),
                o.clone(),
                o.clone(),
            ],
            origin_block_txo_amount as u64,
            &vec![],
            &mut rng,
        );

        let service = setup_wallet_service(ledger_db.clone(), logger.clone());
        let wallet_db = &service.wallet_db;

        // Import the account (this will start it syncing, but we can still process_txos
        // again below for the test)
        let account = service
            .import_account_from_legacy_root_entropy(
                hex::encode(&entropy.bytes),
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .expect("Could not import account entropy");

        // Process the Txos for the first block
        let subaddress_to_txo_ids = process_txos(
            &wallet_db.get_conn().unwrap(),
            &ledger_db.get_block_data(0).unwrap().contents().outputs,
            &account,
            0,
            &logger,
        )
        .expect("could not process txos");

        assert_eq!(subaddress_to_txo_ids.len(), 1);
        assert_eq!(subaddress_to_txo_ids[&0].len(), 16);

        // There should now be 16 txos. Let's get each one and verify the amount
        let expected_value: u64 = 15_625_000 * MOB as u64;
        for txo_id in subaddress_to_txo_ids[&0].clone() {
            let txo = Txo::get(&txo_id, &wallet_db.get_conn().unwrap()).expect("Could not get txo");
            assert_eq!(txo.value as u64, expected_value);
        }

        // Now verify that the service gets the balance with the correct value
        let balance = service
            .get_balance_for_account(&AccountID::from(&account_key))
            .expect("Could not get balance");
        assert_eq!(balance.unspent, 250_000_000 * MOB as u128);
    }
}
