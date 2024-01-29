// Copyright (c) 2018-2023 MobileCoin, Inc.

//! Ledger syncing via the Validator Service.

use mc_blockchain_types::BlockData;
use mc_common::logger::{log, Logger};
use mc_ledger_db::{Ledger, LedgerDB};
use mc_ledger_sync::{NetworkState, PollingNetworkState};
use mc_validator_api::ValidatorUri;
use mc_validator_connection::ValidatorConnection;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
    thread,
    time::Duration,
};

/// The maximum number of blocks to try and retrieve in each iteration
pub const MAX_BLOCKS_PER_SYNC_ITERATION: u32 = 1000;

pub struct ValidatorLedgerSyncThread {
    join_handle: Option<thread::JoinHandle<()>>,
    stop_requested: Arc<AtomicBool>,
}

impl ValidatorLedgerSyncThread {
    pub fn new(
        validator_uri: &ValidatorUri,
        chain_id: String,
        poll_interval: Duration,
        ledger_db: LedgerDB,
        network_state: Arc<RwLock<PollingNetworkState<ValidatorConnection>>>,
        logger: Logger,
    ) -> Self {
        let stop_requested = Arc::new(AtomicBool::new(false));

        let validator_conn = ValidatorConnection::new(validator_uri, chain_id, logger.clone());

        let thread_stop_requested = stop_requested.clone();
        let join_handle = Some(
            thread::Builder::new()
                .name("ValidatorLedgerSync".into())
                .spawn(move || {
                    Self::thread_entrypoint(
                        validator_conn,
                        poll_interval,
                        ledger_db,
                        network_state,
                        logger,
                        thread_stop_requested,
                    );
                })
                .expect("Failed spawning ValidatorLedgerSync thread"),
        );

        Self {
            join_handle,
            stop_requested,
        }
    }

    pub fn stop(&mut self) {
        self.stop_requested.store(true, Ordering::SeqCst);
        if let Some(thread) = self.join_handle.take() {
            thread.join().expect("thread join failed");
        }
    }

    fn thread_entrypoint(
        validator_conn: ValidatorConnection,
        poll_interval: Duration,
        mut ledger_db: LedgerDB,
        mut network_state: Arc<RwLock<PollingNetworkState<ValidatorConnection>>>,
        logger: Logger,
        stop_requested: Arc<AtomicBool>,
    ) {
        log::info!(logger, "ValidatorLedgerSync thread started");

        loop {
            if stop_requested.load(Ordering::SeqCst) {
                log::debug!(logger, "ValidatorLedgerSyncThread stop requested.");
                break;
            }

            let block_data =
                Self::get_next_blocks(&ledger_db, &validator_conn, &mut network_state, &logger);
            if !block_data.is_empty() {
                Self::append_safe_blocks(&mut ledger_db, &block_data, &logger);
            }

            // If we got no blocks, or less than the amount we asked for, sleep for a bit.
            // Getting less the amount we asked for indicates we are fully synced.
            if block_data.is_empty() || block_data.len() < MAX_BLOCKS_PER_SYNC_ITERATION as usize {
                thread::sleep(poll_interval);
            }
        }
    }

    fn get_next_blocks(
        ledger_db: &LedgerDB,
        validator_conn: &ValidatorConnection,
        network_state: &Arc<RwLock<PollingNetworkState<ValidatorConnection>>>,
        logger: &Logger,
    ) -> Vec<BlockData> {
        let num_blocks = ledger_db
            .num_blocks()
            .expect("Failed getting the number of blocks in ledger");

        let (highest_block_index_on_network, is_behind) = {
            let mut network_state = network_state.write().expect("network_state lock poisoned");
            network_state.poll();
            (
                network_state
                    .highest_block_index_on_network()
                    .unwrap_or_default(),
                network_state.is_behind(num_blocks - 1),
            )
        };

        log::trace!(
            logger,
            "local ledger has {} blocks, network highest block index is {}, is_behind:{}",
            num_blocks,
            highest_block_index_on_network,
            is_behind
        );
        if !is_behind {
            return Vec::new();
        }

        log::debug!(logger, "network state is behind, local ledger has {} blocks, network highest block index is {}", num_blocks, highest_block_index_on_network);
        let blocks_data =
            match validator_conn.get_blocks_data(num_blocks, MAX_BLOCKS_PER_SYNC_ITERATION) {
                Ok(blocks_data) => blocks_data,
                Err(err) => {
                    log::error!(
                        logger,
                        "Failed getting blocks data from validator: {:?}",
                        err
                    );
                    return Vec::new();
                }
            };

        mc_ledger_sync::identify_safe_blocks(ledger_db, &blocks_data, logger)
    }

    fn append_safe_blocks(ledger_db: &mut LedgerDB, block_data: &[BlockData], logger: &Logger) {
        log::info!(
            logger,
            "Appending {} blocks to ledger, which currently has {} blocks",
            block_data.len(),
            ledger_db
                .num_blocks()
                .expect("failed getting number of blocks"),
        );

        for block_data in block_data {
            ledger_db
                .append_block(
                    block_data.block(),
                    block_data.contents(),
                    None,
                    block_data.metadata(),
                )
                .unwrap_or_else(|err| {
                    panic!(
                        "Failed appending block #{} to ledger: {}",
                        block_data.block().index,
                        err
                    )
                });
        }
    }
}

impl Drop for ValidatorLedgerSyncThread {
    fn drop(&mut self) {
        self.stop();
    }
}
