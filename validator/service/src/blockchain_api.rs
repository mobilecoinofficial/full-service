// Copyright (c) 2018-2023 MobileCoin, Inc.

//! Blockchain API GRPC service implementation.

use grpcio::{RpcContext, RpcStatus, Service, UnarySink};
use mc_common::logger::Logger;
use mc_connection::{
    BlockchainConnection, ConnectionManager, RetryableBlockchainConnection,
    _retry::delay::Fibonacci,
};
use mc_ledger_db::LedgerDB;
use mc_transaction_core::{tokens::Mob, Token};
use mc_util_grpc::{rpc_logger, rpc_precondition_error, send_result};
use mc_validator_api::{
    consensus_common::{BlocksRequest, BlocksResponse, LastBlockInfoResponse},
    consensus_common_grpc::{create_blockchain_api, BlockchainApi as GrpcBlockchainApi},
    empty::Empty,
};
use rayon::prelude::*; // For par_iter
use std::{collections::HashMap, iter::FromIterator};

pub struct BlockchainApi<BC: BlockchainConnection + 'static> {
    /// Ledger DB.
    ledger_db: LedgerDB,

    /// Connection manager.
    conn_manager: ConnectionManager<BC>,

    /// Logger.
    logger: Logger,
}

impl<BC: BlockchainConnection + 'static> Clone for BlockchainApi<BC> {
    fn clone(&self) -> Self {
        Self {
            ledger_db: self.ledger_db.clone(),
            conn_manager: self.conn_manager.clone(),
            logger: self.logger.clone(),
        }
    }
}

impl<BC: BlockchainConnection + 'static> BlockchainApi<BC> {
    pub fn new(ledger_db: LedgerDB, conn_manager: ConnectionManager<BC>, logger: Logger) -> Self {
        Self {
            ledger_db,
            conn_manager,
            logger,
        }
    }

    pub fn into_service(self) -> Service {
        create_blockchain_api(self)
    }

    fn get_last_block_info_impl(
        &self,
        logger: &Logger,
    ) -> Result<LastBlockInfoResponse, RpcStatus> {
        // Get the last block information from all nodes we are aware of, in parallel.
        let last_block_infos = self
            .conn_manager
            .conns()
            .par_iter()
            .filter_map(|conn| {
                conn.fetch_block_info(Fibonacci::from_millis(10).take(5))
                    .ok()
            })
            .collect::<Vec<_>>();

        // Must have at least one node to get the last block info from.
        let latest_network_block = last_block_infos.first().ok_or_else(|| {
            rpc_precondition_error(
                "last_block_infos",
                "No last block information available",
                logger,
            )
        })?;

        // Ensure that all nodes agree on the last block information.
        if last_block_infos
            .windows(2)
            .any(|window| window[0] != window[1])
        {
            return Err(rpc_precondition_error(
                "minimum_fees",
                "Some nodes do not agree on the last block infos, please wait for them to sync",
                logger,
            ));
        }

        let mut resp = LastBlockInfoResponse::new();

        resp.set_index(latest_network_block.block_index);
        resp.set_network_block_version(latest_network_block.network_block_version);

        // Use minimum fee information from the network (which we previously verified
        // all nodes agree on).
        resp.set_mob_minimum_fee(
            latest_network_block
                .minimum_fee_or_none(&Mob::ID)
                .unwrap_or(Mob::MINIMUM_FEE),
        );
        resp.set_minimum_fees(HashMap::from_iter(
            latest_network_block
                .minimum_fees
                .iter()
                .map(|(token_id, fee)| (**token_id, *fee)),
        ));

        Ok(resp)
    }
}

impl<BC: BlockchainConnection + 'static> GrpcBlockchainApi for BlockchainApi<BC> {
    fn get_last_block_info(
        &mut self,
        ctx: RpcContext,
        _request: Empty,
        sink: UnarySink<LastBlockInfoResponse>,
    ) {
        mc_common::logger::scoped_global_logger(&rpc_logger(&ctx, &self.logger), |logger| {
            send_result(ctx, sink, self.get_last_block_info_impl(logger), logger)
        })
    }

    fn get_blocks(
        &mut self,
        _ctx: RpcContext,
        _req: BlocksRequest,
        _sink: grpcio::UnarySink<BlocksResponse>,
    ) {
    }
}
