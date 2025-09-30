// Copyright (c) 2018-2023 MobileCoin, Inc.

//! Validator API GRPC service implementation.

use grpcio::{EnvBuilder, RpcContext, RpcStatus, Service, UnarySink};
use mc_common::logger::{log, Logger};
use mc_connection::{
    ConnectionManager, Error as ConnectionError, RetryError, RetryableUserTxConnection,
    UserTxConnection, _retry::delay::Fibonacci,
};
use mc_fog_report_connection::{Error as FogConnectionError, GrpcFogReportConnection};
use mc_ledger_db::{Ledger, LedgerDB};
use mc_util_grpc::{
    check_request_chain_id, rpc_database_err, rpc_internal_error, rpc_invalid_arg_error,
    rpc_logger, rpc_permissions_error, send_result,
};
use mc_util_uri::FogUri;
use mc_validator_api::{
    blockchain::ArchiveBlocks,
    consensus_common::{BlocksRequest, ProposeTxResponse},
    external::Tx,
    validator_api::{
        create_validator_api, FetchFogReportRequest, FetchFogReportResponse, FetchFogReportResult,
        ValidatorApi as GrpcValidatorApi,
    },
};
use std::{
    convert::TryFrom,
    str::FromStr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

/// Maximal number of blocks we will return in a single request.
pub const MAX_BLOCKS_PER_REQUEST: u32 = 1000;

pub struct ValidatorApi<UTC: UserTxConnection + 'static> {
    /// Ledger DB.
    ledger_db: LedgerDB,

    /// Connection manager.
    conn_manager: ConnectionManager<UTC>,

    /// Monotonically increasing counter. This is used for node round-robin
    /// selection.
    submit_node_offset: Arc<AtomicUsize>,

    /// Fog report connection.
    fog_report_connection: GrpcFogReportConnection,

    /// Chain id.
    chain_id: String,

    /// Logger.
    logger: Logger,
}

impl<UTC: UserTxConnection + 'static> Clone for ValidatorApi<UTC> {
    fn clone(&self) -> Self {
        Self {
            ledger_db: self.ledger_db.clone(),
            conn_manager: self.conn_manager.clone(),
            submit_node_offset: self.submit_node_offset.clone(),
            fog_report_connection: self.fog_report_connection.clone(),
            chain_id: self.chain_id.clone(),
            logger: self.logger.clone(),
        }
    }
}

impl<UTC: UserTxConnection + 'static> ValidatorApi<UTC> {
    pub fn new(
        chain_id: String,
        ledger_db: LedgerDB,
        conn_manager: ConnectionManager<UTC>,
        logger: Logger,
    ) -> Self {
        Self {
            ledger_db,
            conn_manager,
            submit_node_offset: Arc::new(AtomicUsize::new(0)),
            fog_report_connection: GrpcFogReportConnection::new(
                chain_id.clone(),
                Arc::new(
                    EnvBuilder::new()
                        .name_prefix("FogReportGrpc".to_string())
                        .build(),
                ),
                logger.clone(),
            ),
            chain_id,
            logger,
        }
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

    fn propose_tx_impl(&self, tx: Tx, logger: &Logger) -> Result<ProposeTxResponse, RpcStatus> {
        // Convert Protobuf/GRPC Tx to Prost Tx
        let tx = mc_transaction_core::tx::Tx::try_from(&tx)
            .map_err(|_| rpc_invalid_arg_error("propose_tx", "tx", logger))?;

        // Figure out which node to submit to
        let responder_ids = self.conn_manager.responder_ids();
        if responder_ids.is_empty() {
            return Err(rpc_internal_error("propose_tx", "no peers", logger));
        }

        let idx = self.submit_node_offset.fetch_add(1, Ordering::SeqCst);
        let responder_id = &responder_ids[idx % responder_ids.len()];

        // Submit.
        let tx_propose_result = self
            .conn_manager
            .conn(responder_id)
            .ok_or_else(|| rpc_internal_error("propose_tx", "conn not found", logger))?
            .propose_tx(&tx, Fibonacci::from_millis(10).take(5));

        // Convert to GRPC response.
        match tx_propose_result {
            Ok(block_count) => Ok(ProposeTxResponse {
                block_count,
                ..Default::default()
            }),

            Err(RetryError { error, .. }) => match error {
                err @ ConnectionError::Cipher(_) => {
                    Err(rpc_permissions_error("propose_tx", err, logger))
                }
                ConnectionError::Attestation(_) => {
                    Err(rpc_permissions_error("propose_tx", error, logger))
                }
                ConnectionError::TransactionValidation(err, _) => Ok(ProposeTxResponse {
                    result: err.into(),
                    ..Default::default()
                }),
                err => Err(rpc_internal_error("propose_tx", format!("{err:?}"), logger)),
            },
        }
    }

    fn fetch_fog_report_impl(
        &self,
        request: FetchFogReportRequest,
        logger: &Logger,
    ) -> Result<FetchFogReportResponse, RpcStatus> {
        let fog_uri = FogUri::from_str(&request.uri)
            .map_err(|_| rpc_invalid_arg_error("fetch_fog_report", "uri", logger))?;

        match self.fog_report_connection.fetch_fog_report(&fog_uri) {
            Ok(report_response) => Ok(FetchFogReportResponse {
                result: FetchFogReportResult::Ok as i32,
                report: Some(report_response.into()),
            }),

            Err(FogConnectionError::NoReports(_)) => Ok(FetchFogReportResponse {
                result: FetchFogReportResult::NoReports as i32,
                report: None,
            }),

            // TODO do we want a special case for
            // Err(FogConnectionError::Grpc(Error::RpcFailure(status_code))) ?
            err => Err(rpc_internal_error(
                "fetch_fog_report",
                format!("{err:?}"),
                logger,
            )),
        }
    }

    /// Check the chain-id, if available.
    fn maybe_check_request_chain_id(&self, ctx: &RpcContext) -> Result<(), RpcStatus> {
        if self.chain_id.is_empty() {
            return Ok(());
        }

        check_request_chain_id(&self.chain_id, ctx)
    }
}

impl<UTC: UserTxConnection + 'static> GrpcValidatorApi for ValidatorApi<UTC> {
    fn get_archive_blocks(
        &mut self,
        ctx: RpcContext,
        request: BlocksRequest,
        sink: UnarySink<ArchiveBlocks>,
    ) {
        mc_common::logger::scoped_global_logger(&rpc_logger(&ctx, &self.logger), |logger| {
            if let Err(err) = self.maybe_check_request_chain_id(&ctx) {
                return send_result(ctx, sink, Err(err), &self.logger);
            }

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
            if let Err(err) = self.maybe_check_request_chain_id(&ctx) {
                return send_result(ctx, sink, Err(err), &self.logger);
            }

            send_result(ctx, sink, self.propose_tx_impl(request, logger), logger)
        })
    }

    fn fetch_fog_report(
        &mut self,
        ctx: RpcContext,
        request: FetchFogReportRequest,
        sink: UnarySink<FetchFogReportResponse>,
    ) {
        mc_common::logger::scoped_global_logger(&rpc_logger(&ctx, &self.logger), |logger| {
            if let Err(err) = self.maybe_check_request_chain_id(&ctx) {
                return send_result(ctx, sink, Err(err), &self.logger);
            }

            send_result(
                ctx,
                sink,
                self.fetch_fog_report_impl(request, logger),
                logger,
            )
        })
    }
}
