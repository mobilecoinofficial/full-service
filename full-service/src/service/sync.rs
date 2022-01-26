// Copyright (c) 2018-2020 MobileCoin Inc.

//! Manages ledger block scanning for wallet accounts.

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
use mc_crypto_keys::{RistrettoPrivate, RistrettoPublic};
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
        Arc, Mutex,
    },
    thread,
    time::Duration,
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

                        sync_all_accounts(&ledger_db, &wallet_db, &logger).unwrap(); // TODO: error logging

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

    dbg!(num_blocks);

    // Go over our list of accounts and see which ones need to process more blocks.
    let accounts = {
        let conn = &wallet_db
            .get_conn()
            .expect("Could not get connection to DB");
        conn.transaction::<Vec<Account>, WalletDbError, _>(|| {
            Ok(Account::list_all(conn).expect("Failed getting accounts from database"))
        })
        .expect("Failed executing database transaction")
    };
    for account in accounts {
        // If there are no new blocks for this account, don't do anything.
        if account.next_block_index >= num_blocks as i64 {
            continue;
        }
        sync_account(&ledger_db, &wallet_db, &account.account_id_hex, &logger)?;
    }

    Ok(())
}

/// Sync a single account.
fn sync_account(
    ledger_db: &LedgerDB,
    wallet_db: &WalletDb,
    account_id_hex: &str,
    logger: &Logger,
) -> Result<(), SyncError> {
    let conn = wallet_db.get_conn()?;
    loop {
        // Sync one chunk of blocks for this account each iteration of the loop.
        conn.transaction::<(), SyncError, _>(|| {
            // Get the account data. If it is no longer available, the account has been
            // removed and we can simply return.
            let account = Account::get(&AccountID(account_id_hex.to_string()), &conn)?;
            let account_key: AccountKey = mc_util_serial::decode(&account.account_key)?;

            // Load subaddresses for this account into a hash map.
            let mut subaddress_keys: HashMap<RistrettoPublic, u64> = HashMap::default();
            let subaddresses: Vec<_> =
                AssignedSubaddress::list_all(account_id_hex, None, None, &conn)?;
            for s in subaddresses {
                let subaddress_key = mc_util_serial::decode(s.subaddress_spend_key.as_slice())?;
                subaddress_keys.insert(subaddress_key, s.subaddress_index as u64);
            }

            use std::time::{Duration, Instant};
            let start_time = Instant::now();
            let first_block_index = account.next_block_index;
            let mut last_block_index = account.next_block_index;

            // Load transaction outputs for this chunk.
            let mut tx_outs: Vec<_> = Vec::new();
            for block_index in
                (account.next_block_index..account.next_block_index + BLOCKS_CHUNK_SIZE)
            {
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
            }

            // Attempt to decode each transaction as received by this account.
            let matched_txos: Vec<_> = tx_outs
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

            let duration = start_time.elapsed();
            log::trace!(
                logger,
                "Synced {} blocks ({}-{}) for account {} in {:?}. {} txos found.",
                last_block_index - first_block_index + 1,
                first_block_index,
                last_block_index,
                account_id_hex.chars().take(6).collect::<String>(),
                duration,
                matched_txos.len(),
            );

            account.update_next_block_index(
                account.next_block_index,
                block_contents.key_images,
                &conn,
            )?;

            // Write matched transactions to the database.
            for (block_index, tx_out, amount, subaddress_index, key_image) in matched_txos {
                let txo_id = Txo::create_received(
                    tx_out.clone(),
                    subaddress_index.map(|i| i as i64),
                    key_image,
                    amount,
                    block_index,
                    &account_id_hex,
                    &conn,
                )?;
            }

            // // Add a transaction for the received TXOs
            // TransactionLog::log_received(
            //     &output_txo_ids,
            //     &account,
            //     account.next_block_index as u64,
            //     &conn,
            // )?;

            Ok(())
        })?;
    }
}

/// Attempt to decode the transaction amount. If we can't, then this transaction
/// does not belong to this account.
pub fn decode_amount(tx_out: &TxOut, account_key: &AccountKey) -> Option<u64> {
    let tx_public_key = match RistrettoPublic::try_from(&tx_out.public_key) {
        Err(_) => return None,
        Ok(k) => k,
    };
    let shared_secret = get_tx_out_shared_secret(account_key.view_private_key(), &tx_public_key);
    let amount = match tx_out.amount.get_value(&shared_secret) {
        Ok((a, _)) => Some(a),
        Err(AmountError::InconsistentCommitment) => return None,
    };
    amount
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
