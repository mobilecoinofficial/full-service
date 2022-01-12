// Copyright (c) 2018-2022 MobileCoin, Inc.

//! Validator API GRPC service implementation.

use grpcio::{RpcContext, RpcStatus, Service, UnarySink};
use mc_common::logger::{log, Logger};
use mc_ledger_db::{Ledger, LedgerDB};
use mc_util_grpc::{rpc_database_err, rpc_invalid_arg_error, rpc_logger, send_result};
use mc_validator_api::{
    blockchain::ArchiveBlocks,
    consensus_common::{BlocksRequest, ProposeTxResponse},
    external::Tx,
    report::ReportResponse,
    validator_api::FetchFogReportRequest,
    validator_api_grpc::{create_validator_api, ValidatorApi as GrpcValidatorApi},
};

/// Maximal number of blocks we will return in a single request.
pub const MAX_BLOCKS_PER_REQUEST: u32 = 1000;

#[derive(Clone)]
pub struct ValidatorApi {
    /// Ledger DB.
    ledger_db: LedgerDB,

    /// Logger.
    logger: Logger,
}

impl ValidatorApi {
    pub fn new(ledger_db: LedgerDB, logger: Logger) -> Self {
        Self { ledger_db, logger }
    }

    pub fn into_service(self) -> Service {
        create_validator_api(self)
    }

    fn get_archive_blocks_impl(
        &self,
        request: BlocksRequest,
        logger: &Logger,
    ) -> Result<ArchiveBlocks, RpcStatus> {
        log::trace!(
            logger,
            "get_archive_blocks(offset:{}, limit:{})",
            request.offset,
            request.limit
        );

        if request.limit > MAX_BLOCKS_PER_REQUEST {
            return Err(rpc_invalid_arg_error("get_archive_blocks", "limit", logger));
        }

        let start_index = request.offset;
        let end_index = request.offset + request.limit as u64;

        // Get "persistence type" blocks.
        let mut blocks_data = Vec::new();
        for block_index in start_index..end_index {
            match self.ledger_db.get_block_data(block_index) {
                Ok(block_data) => blocks_data.push(block_data),
                Err(mc_ledger_db::Error::NotFound) => {
                    // This is okay - it means we have reached the last block in the ledger in the
                    // previous loop iteration.
                    break;
                }
                Err(error) => {
                    log::error!(logger, "Error getting block {}: {:?}", block_index, error);
                    return Err(rpc_database_err(error, logger));
                }
            }
        }

        Ok(ArchiveBlocks::from(&blocks_data[..]))
    }

    fn propose_tx_impl(
        &self,
        _request: Tx,
        _logger: &Logger,
    ) -> Result<ProposeTxResponse, RpcStatus> {
        todo!()
    }

    fn fetch_fog_report_impl(
        &self,
        _request: FetchFogReportRequest,
        _logger: &Logger,
    ) -> Result<ReportResponse, RpcStatus> {
        todo!()
    }
}

impl GrpcValidatorApi for ValidatorApi {
    fn get_archive_blocks(
        &mut self,
        ctx: RpcContext,
        request: BlocksRequest,
        sink: UnarySink<ArchiveBlocks>,
    ) {
        mc_common::logger::scoped_global_logger(&rpc_logger(&ctx, &self.logger), |logger| {
            send_result(
                ctx,
                sink,
                self.get_archive_blocks_impl(request, logger),
                logger,
            )
        })
    }

    fn propose_tx(&mut self, ctx: RpcContext, request: Tx, sink: UnarySink<ProposeTxResponse>) {
        mc_common::logger::scoped_global_logger(&rpc_logger(&ctx, &self.logger), |logger| {
            send_result(ctx, sink, self.propose_tx_impl(request, logger), logger)
        })
    }

    fn fetch_fog_report(
        &mut self,
        ctx: RpcContext,
        request: FetchFogReportRequest,
        sink: UnarySink<ReportResponse>,
    ) {
        mc_common::logger::scoped_global_logger(&rpc_logger(&ctx, &self.logger), |logger| {
            send_result(
                ctx,
                sink,
                self.fetch_fog_report_impl(request, logger),
                logger,
            )
        })
    }
}
