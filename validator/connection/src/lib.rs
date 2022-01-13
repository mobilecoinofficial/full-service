// Copyright (c) 2018-2022 MobileCoin, Inc.

//! Validator GRPC client.

mod error;

use grpcio::{ChannelBuilder, EnvBuilder};
use mc_common::logger::Logger;
use mc_transaction_core::BlockData;
use mc_util_grpc::ConnectionUriGrpcioChannel;
use mc_validator_api::{
    blockchain::ArchiveBlock, consensus_common::BlocksRequest,
    consensus_common_grpc::BlockchainApiClient, validator_api_grpc::ValidatorApiClient,
    ValidatorUri,
};
use std::{convert::TryFrom, sync::Arc};

pub use error::Error;

#[derive(Clone)]
pub struct ValidatorConnection {
    validator_api_client: ValidatorApiClient,
    blockchain_api_client: BlockchainApiClient,
}

impl ValidatorConnection {
    pub fn new(uri: &ValidatorUri, logger: Logger) -> Self {
        let env = Arc::new(EnvBuilder::new().name_prefix("ValidatorRPC").build());
        let ch = ChannelBuilder::new(env)
            .max_receive_message_len(std::i32::MAX)
            .max_send_message_len(std::i32::MAX)
            .connect_to_uri(uri, &logger);

        let validator_api_client = ValidatorApiClient::new(ch.clone());
        let blockchain_api_client = BlockchainApiClient::new(ch);

        Self {
            validator_api_client,
            blockchain_api_client,
        }
    }

    pub fn get_archive_blocks(&self, offset: u64, limit: u32) -> Result<Vec<ArchiveBlock>, Error> {
        let mut request = BlocksRequest::new();
        request.set_offset(offset);
        request.set_limit(limit);

        let response = self.validator_api_client.get_archive_blocks(&request)?;

        Ok(response.get_blocks().to_vec())
    }

    pub fn get_blocks_data(&self, offset: u64, limit: u32) -> Result<Vec<BlockData>, Error> {
        let archive_blocks = self.get_archive_blocks(offset, limit)?;

        let blocks_data = archive_blocks
            .iter()
            .map(BlockData::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(blocks_data)
    }
}
