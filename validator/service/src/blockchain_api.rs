// Copyright (c) 2018-2022 MobileCoin, Inc.

//! Blockchain API GRPC service implementation.

use grpcio::{RpcContext, RpcStatus, Service, UnarySink};
use mc_common::logger::Logger;
use mc_util_grpc::{rpc_logger, send_result};
use mc_validator_api::{
    consensus_common::LastBlockInfoResponse,
    consensus_common_grpc::{create_blockchain_api, BlockchainApi as GrpcBlockchainApi},
    empty::Empty,
};

#[derive(Clone)]
pub struct BlockchainApi {
    logger: Logger,
}

impl BlockchainApi {
    pub fn new(logger: Logger) -> Self {
        Self { logger }
    }

    pub fn into_service(self) -> Service {
        create_blockchain_api(self)
    }

    fn get_last_block_info_impl(
        &self,
        _logger: &Logger,
    ) -> Result<LastBlockInfoResponse, RpcStatus> {
        todo!()
    }
}

impl GrpcBlockchainApi for BlockchainApi {
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

    // TODO: GetBlocks is purposefully unimplemented since it is unclear if it will
    // be needed.
}
