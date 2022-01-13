// Copyright (c) 2018-2022 MobileCoin, Inc.

//! Validator GRPC client.

mod error;

use grpcio::{ChannelBuilder, EnvBuilder};
use mc_common::logger::Logger;
use mc_connection::{
    BlockInfo, BlockchainConnection, Connection, Error as ConnectionError,
    Result as ConnectionResult, UserTxConnection,
};
use mc_transaction_core::{tx::Tx, Block, BlockData, BlockID, BlockIndex};
use mc_util_grpc::ConnectionUriGrpcioChannel;
use mc_util_uri::ConnectionUri;
use mc_validator_api::{
    blockchain::ArchiveBlock,
    consensus_common::{BlocksRequest, ProposeTxResult},
    consensus_common_grpc::BlockchainApiClient,
    empty::Empty,
    validator_api_grpc::ValidatorApiClient,
    ValidatorUri,
};
use std::{
    cmp::Ordering,
    convert::TryFrom,
    fmt::{Display, Formatter, Result as FmtResult},
    hash::{Hash, Hasher},
    ops::Range,
    sync::Arc,
};

pub use error::Error;

#[derive(Clone)]
pub struct ValidatorConnection {
    uri: ValidatorUri,
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
            uri: uri.clone(),
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

impl Display for ValidatorConnection {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.uri)
    }
}

impl Eq for ValidatorConnection {}

impl Hash for ValidatorConnection {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.uri.addr().hash(hasher);
    }
}

impl PartialEq for ValidatorConnection {
    fn eq(&self, other: &Self) -> bool {
        self.uri.addr() == other.uri.addr()
    }
}

impl Ord for ValidatorConnection {
    fn cmp(&self, other: &Self) -> Ordering {
        self.uri.addr().cmp(&other.uri.addr())
    }
}

impl PartialOrd for ValidatorConnection {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.uri.addr().partial_cmp(&other.uri.addr())
    }
}

impl Connection for ValidatorConnection {
    type Uri = ValidatorUri;

    fn uri(&self) -> Self::Uri {
        self.uri.clone()
    }
}

impl BlockchainConnection for ValidatorConnection {
    /// Retrieve the block metadata from the blockchain service.
    fn fetch_blocks(&mut self, range: Range<BlockIndex>) -> ConnectionResult<Vec<Block>> {
        let limit =
            u32::try_from(range.end - range.start).or(Err(ConnectionError::RequestTooLarge))?;
        let blocks_data = self.get_blocks_data(range.start, limit)?;

        Ok(blocks_data
            .into_iter()
            .map(|block_data| block_data.block().clone())
            .collect())
    }

    /// Retrieve the BlockIDs (hashes) of the given blocks from the blockchain
    /// service.
    fn fetch_block_ids(&mut self, range: Range<BlockIndex>) -> ConnectionResult<Vec<BlockID>> {
        self.fetch_blocks(range)
            .map(|blocks| blocks.into_iter().map(|block| block.id).collect())
    }

    /// Retrieve the consensus node's current block height
    fn fetch_block_height(&mut self) -> ConnectionResult<BlockIndex> {
        let response = self
            .blockchain_api_client
            .get_last_block_info(&Empty::new())?;
        Ok(response.get_index())
    }

    /// Retrieve the consensus node's current block height and fee
    fn fetch_block_info(&mut self) -> ConnectionResult<BlockInfo> {
        let response = self
            .blockchain_api_client
            .get_last_block_info(&Empty::new())?;
        Ok(response.into())
    }
}

impl UserTxConnection for ValidatorConnection {
    fn propose_tx(&mut self, tx: &Tx) -> ConnectionResult<u64> {
        let response = self.validator_api_client.propose_tx(&tx.into())?;
        if response.get_result() == ProposeTxResult::Ok {
            Ok(response.get_block_count())
        } else {
            Err(response.get_result().into())
        }
    }
}
