// Copyright (c) 2018-2020 MobileCoin Inc.

//! Manages ledger block scanning for wallet accounts.

use crate::{
    db::{
        account::{AccountID, AccountModel},
        assigned_subaddress::AssignedSubaddressModel,
        models::{Account, AssignedSubaddress, TransactionLog, Txo},
        transaction_log::TransactionLogModel,
        txo::TxoModel,
        WalletDb,
    },
    error::SyncError,
    util::b58::b58_encode_public_address,
};
use mc_account_keys::AccountKey;
use mc_common::{
    logger::{log, Logger},
    HashMap,
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
use rayon::prelude::*;

use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, PooledConnection},
};
use std::{
    convert::TryFrom,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Instant,
};

const BLOCKS_CHUNK_SIZE: i64 = 10_000;

/// Sync thread - holds objects needed to cleanly terminate the sync thread.
pub struct SyncThread {
    /// The main sync thread handle.
    join_handle: Option<thread::JoinHandle<()>>,

    /// Stop trigger, used to signal the thread to terminate.
    stop_requested: Arc<AtomicBool>,
}

impl SyncThread {
    pub fn start(ledger_db: LedgerDB, wallet_db: WalletDb, logger: Logger) -> Self {
        // Start the sync thread.

        let stop_requested = Arc::new(AtomicBool::new(false));
        let thread_stop_requested = stop_requested.clone();

        let join_handle = Some(
            thread::Builder::new()
                .name("sync".to_string())
                .spawn(move || {
                    log::debug!(logger, "Sync thread started.");

                    loop {
                        if thread_stop_requested.load(Ordering::SeqCst) {
                            log::debug!(logger, "SyncThread stop requested.");
                            break;
                        }
                        match sync_all_accounts(&ledger_db, &wallet_db, &logger) {
                            Ok(()) => (),
                            Err(e) => log::error!(&logger, "Error during account sync:\n{:?}", e),
                        }

                        thread::sleep(std::time::Duration::from_secs(1));
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

pub fn sync_all_accounts(
    ledger_db: &LedgerDB,
    wallet_db: &WalletDb,
    logger: &Logger,
) -> Result<(), SyncError> {
    // Get the current number of blocks in ledger.
    let num_blocks = ledger_db
        .num_blocks()
        .expect("failed getting number of blocks");

    // Go over our list of accounts and see which ones need to process more blocks.
    let accounts = {
        let conn = &wallet_db
            .get_conn()
            .expect("Could not get connection to DB");
        Account::list_all(conn).expect("Failed getting accounts from database")
    };

    for account in accounts {
        // If there are no new blocks for this account, don't do anything.
        if account.next_block_index as u64 >= num_blocks - 1 {
            continue;
        }
        sync_account(ledger_db, wallet_db, &account.account_id_hex, logger)?;
    }

    Ok(())
}

#[derive(Debug)]
enum SyncStatus {
    ChunkFinished,
    NoMoreBlocks,
}

/// Sync a single account.
pub fn sync_account(
    ledger_db: &LedgerDB,
    wallet_db: &WalletDb,
    account_id_hex: &str,
    logger: &Logger,
) -> Result<(), SyncError> {
    let conn = wallet_db.get_conn()?;

    while let SyncStatus::ChunkFinished =
        sync_account_next_chunk(ledger_db, &conn, logger, account_id_hex)?
    {}

    Ok(())
}

fn sync_account_next_chunk(
    ledger_db: &LedgerDB,
    conn: &PooledConnection<ConnectionManager<SqliteConnection>>,
    logger: &Logger,
    account_id_hex: &str,
) -> Result<SyncStatus, SyncError> {
    conn.transaction::<SyncStatus, SyncError, _>(|| {
        // Get the account data. If it is no longer available, the account has been
        // removed and we can simply return.
        let account = Account::get(&AccountID(account_id_hex.to_string()), conn)?;
        let account_key: AccountKey = mc_util_serial::decode(&account.account_key)?;

        // Load subaddresses for this account into a hash map.
        let mut subaddress_keys: HashMap<RistrettoPublic, u64> = HashMap::default();
        let subaddresses: Vec<_> = AssignedSubaddress::list_all(account_id_hex, None, None, conn)?;
        for s in subaddresses {
            let subaddress_key = mc_util_serial::decode(s.subaddress_spend_key.as_slice())?;
            subaddress_keys.insert(subaddress_key, s.subaddress_index as u64);
        }

        let start_time = Instant::now();
        let first_block_index = account.next_block_index;
        let mut last_block_index = account.next_block_index;

        // Load transaction outputs and key images for this chunk.
        let mut tx_outs: Vec<(i64, TxOut)> = Vec::new();
        let mut key_images: Vec<(i64, KeyImage)> = Vec::new();
        for block_index in account.next_block_index..account.next_block_index + BLOCKS_CHUNK_SIZE {
            let block_contents = match ledger_db.get_block_contents(block_index as u64) {
                Ok(block_contents) => block_contents,
                Err(mc_ledger_db::Error::NotFound) => {
                    break;
                }
                Err(err) => {
                    return Err(err.into());
                }
            };
            last_block_index = block_index;

            for tx_out in block_contents.outputs {
                tx_outs.push((block_index, tx_out));
            }

            for key_image in block_contents.key_images {
                key_images.push((block_index, key_image));
            }
        }
        if tx_outs.is_empty() {
            return Ok(SyncStatus::NoMoreBlocks);
        }

        // Attempt to decode each transaction as received by this account.
        let received_txos: Vec<_> = tx_outs
            .into_par_iter()
            .filter_map(|(block_index, tx_out)| {
                let amount = match decode_amount(&tx_out, &account_key) {
                    None => return None,
                    Some(a) => a,
                };
                let (subaddress_index, key_image) =
                    decode_subaddress_and_key_image(&tx_out, &account_key, &subaddress_keys);
                Some((block_index, tx_out, amount, subaddress_index, key_image))
            })
            .collect();
        let num_received_txos = received_txos.len();

        // Write received transactions to the database.
        for (block_index, tx_out, amount, subaddress_index, key_image) in received_txos {
            let txo_id = Txo::create_received(
                tx_out.clone(),
                subaddress_index.map(|i| i as i64),
                key_image,
                amount,
                block_index,
                account_id_hex,
                conn,
            )?;

            // TODO: What's the best way to get the assigned_subaddress_b58?
            // Do we even care about saving this in the database at all? We
            // should be able to look up any relevant information about the
            // txo directly from the txo table. This will also hinder us
            // from supporting recoverable transaction history in the case that
            // there are txo's that go to multiple different subaddresses in the
            // same transaction.
            // My thoughts are to remove assigned_subaddress_b58 entirely from
            // this table and use the TransactionTxoType table to look up info
            // about each of the txo's independently, since each on could
            // be at a different subaddress.
            // In fact, do we even want to be creating a TransactionLog for
            // individual txo's at all, since all of this information is
            // derivable from the txo's table? The only thing that's necessary
            // to store in the database WRT a transaction is when we send,
            // because that requires extra meta data that isn't derivable
            // from the ledger.
            //
            // TL;DR
            // Reconsider creating a TransactionLog in favor of deriving the
            // information from the txo's table when necessary, and only
            // store information about sent transactions.

            let assigned_subaddress_b58: Option<String> = match subaddress_index {
                None => None,
                Some(subaddress_index) => {
                    let subaddress = account_key.subaddress(subaddress_index);
                    let subaddress_b58 = b58_encode_public_address(&subaddress)?;
                    Some(subaddress_b58)
                }
            };

            TransactionLog::log_received(
                account_id_hex,
                assigned_subaddress_b58.as_deref(),
                txo_id.as_str(),
                amount as i64,
                block_index as u64,
                conn,
            )?;
        }

        // Match key images to mark existing unspent transactions as spent.
        let unspent_key_images: HashMap<KeyImage, String> =
            Txo::list_unspent_key_images(account_id_hex, conn)?;
        let spent_txos: Vec<_> = key_images
            .par_iter()
            .filter_map(|(block_index, key_image)| {
                unspent_key_images
                    .get(key_image)
                    .map(|txo_id_hex| (block_index, txo_id_hex))
            })
            .collect();
        let num_spent_txos = spent_txos.len();
        for (block_index, txo_id_hex) in spent_txos {
            Txo::update_to_spent(txo_id_hex, block_index, conn)?;
            // TODO: Find TransactionLogs with this spent_txo and update it to
            // TX_STATUS_SPENT
        }

        // Done syncing this chunk. Mark these blocks as synced for this account.
        account.update_next_block_index(last_block_index + 1, conn)?;

        // TODO: Find txos with a pending_tombstone_block_index < the last_block_index
        // and set them to spendable.

        // TODO: Find any TransactionLogs associated with these txos and update them
        // to TX_STATUS_FAILED

        let duration = start_time.elapsed();

        log::debug!(
            logger,
            "Synced {} blocks ({}-{}) for account {} in {:?}. {} txos received, {}/{} txos spent.",
            last_block_index - first_block_index + 1,
            first_block_index,
            last_block_index,
            account_id_hex.chars().take(6).collect::<String>(),
            duration,
            num_received_txos,
            num_spent_txos,
            unspent_key_images.len(),
        );

        Ok(SyncStatus::ChunkFinished)
    })
}

/// Attempt to decode the transaction amount. If we can't, then this transaction
/// does not belong to this account.
pub fn decode_amount(tx_out: &TxOut, account_key: &AccountKey) -> Option<u64> {
    let tx_public_key = match RistrettoPublic::try_from(&tx_out.public_key) {
        Err(_) => return None,
        Ok(k) => k,
    };
    let shared_secret = get_tx_out_shared_secret(account_key.view_private_key(), &tx_public_key);
    match tx_out.amount.get_value(&shared_secret) {
        Ok((a, _)) => Some(a),
        Err(AmountError::InconsistentCommitment) => None,
    }
}

/// Attempt to match the target address with one of our subaddresses. This
/// should only be done on tx-outs that have already had their amounts decoded.
/// If this fails, then the transaction is "orphaned", meaning we haven't
/// generated the correct subaddress yet.
pub fn decode_subaddress_and_key_image(
    tx_out: &TxOut,
    account_key: &AccountKey,
    subaddress_keys: &HashMap<RistrettoPublic, u64>,
) -> (Option<u64>, Option<KeyImage>) {
    let tx_public_key = match RistrettoPublic::try_from(&tx_out.public_key) {
        Ok(k) => k,
        Err(_) => return (None, None),
    };
    let tx_out_target_key = match RistrettoPublic::try_from(&tx_out.target_key) {
        Ok(k) => k,
        Err(_) => return (None, None),
    };
    let subaddress_spk: RistrettoPublic = recover_public_subaddress_spend_key(
        account_key.view_private_key(),
        &tx_out_target_key,
        &tx_public_key,
    );
    let subaddress_index = subaddress_keys.get(&subaddress_spk).copied();

    let key_image = if let Some(subaddress_i) = subaddress_index {
        let onetime_private_key = recover_onetime_private_key(
            &tx_public_key,
            account_key.view_private_key(),
            &account_key.subaddress_spend_private(subaddress_i),
        );
        Some(KeyImage::from(&onetime_private_key))
    } else {
        None
    };

    (subaddress_index, key_image)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        service::{account::AccountService, balance::BalanceService, txo::TxoService},
        test_utils::{
            add_block_to_ledger_db, get_test_ledger, manually_sync_account, setup_wallet_service,
            MOB,
        },
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

        // Import the account
        let _account = service
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

        manually_sync_account(
            &ledger_db,
            &wallet_db,
            &AccountID::from(&account_key),
            1,
            &logger,
        );

        // There should now be 16 txos. Let's get each one and verify the amount
        let expected_value: u64 = 15_625_000 * MOB as u64;

        let txos = service
            .list_txos(&AccountID::from(&account_key), None, None)
            .unwrap();

        for txo in txos {
            assert_eq!(txo.value as u64, expected_value);
        }

        // Now verify that the service gets the balance with the correct value
        let balance = service
            .get_balance_for_account(&AccountID::from(&account_key))
            .expect("Could not get balance");
        assert_eq!(balance.unspent, 250_000_000 * MOB as u128);
    }
}
