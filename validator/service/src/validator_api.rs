// Copyright (c) 2018-2022 MobileCoin, Inc.

//! Validator API GRPC service implementation.

use grpcio::{RpcContext, RpcStatus, Service, UnarySink};
use mc_common::logger::Logger;
use mc_ledger_db::LedgerDB;
use mc_util_grpc::{rpc_logger, send_result};
use mc_validator_api::{
    blockchain::ArchiveBlocks,
    consensus_common::{BlocksRequest, ProposeTxResponse},
    external::Tx,
    report::ReportResponse,
    validator_api::FetchFogReportRequest,
    validator_api_grpc::{create_validator_api, ValidatorApi as GrpcValidatorApi},
};

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
        todo!()
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
