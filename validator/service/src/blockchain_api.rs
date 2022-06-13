// Copyright (c) 2018-2022 MobileCoin, Inc.

//! Blockchain API GRPC service implementation.

use grpcio::{RpcContext, RpcStatus, Service, UnarySink};
use mc_common::logger::Logger;
use mc_connection::{BlockchainConnection, ConnectionManager, RetryableBlockchainConnection};
use mc_ledger_db::{Ledger, LedgerDB};
use mc_transaction_core::{tokens::Mob, Token};
use mc_util_grpc::{rpc_database_err, rpc_logger, send_result};
use mc_validator_api::{
    consensus_common::{BlocksRequest, BlocksResponse, LastBlockInfoResponse},
    consensus_common_grpc::{create_blockchain_api, BlockchainApi as GrpcBlockchainApi},
    empty::Empty,
};
use rayon::prelude::*; // For par_iter

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
        let num_blocks = self
            .ledger_db
            .num_blocks()
            .map_err(|err| rpc_database_err(err, logger))?;

        let mut resp = LastBlockInfoResponse::new();
        resp.set_index(num_blocks - 1);

        // Iterate an owned list of connections in parallel, get the block info for
        // each, and extract the fee. If no fees are returned, use the hard-coded
        // minimum.
        let minimum_fee = self
            .conn_manager
            .conns()
            .par_iter()
            .filter_map(|conn| conn.fetch_block_info(std::iter::empty()).ok())
            .filter_map(|block_info| block_info.minimum_fee_or_none(&Mob::ID))
            .max()
            .unwrap_or(Mob::MINIMUM_FEE);
        resp.set_mob_minimum_fee(minimum_fee);

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
