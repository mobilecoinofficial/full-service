mod error;

use mc_common::logger::{log, Logger};
use mc_connection::Connection;
use mc_util_grpc::ConnectionUriGrpcioChannel;
use mc_util_uri::ConnectionUri;
use t3_api::{
    t3_grpc::TransactionServiceClient, CreateTransactionRequest, CreateTransactionResponse,
    FindTransactionsRequest, T3Uri, TestErrorRequest, TransparentTransaction,
};

use grpcio::{CallOption, ChannelBuilder, EnvBuilder, MetadataBuilder};

use std::{
    cmp::Ordering,
    fmt::{Display, Formatter, Result as FmtResult},
    hash::{Hash, Hasher},
    sync::Arc,
};

pub use error::Error;

pub fn common_headers_call_option(api_key: &str) -> CallOption {
    let mut metadata_builder = MetadataBuilder::new();
    let api_key_string = format!("ApiKey {}", api_key);
    metadata_builder
        .add_str("Authorization", &api_key_string)
        .expect("Could not add api-key header");

    CallOption::default().headers(metadata_builder.build())
}

#[derive(Clone)]
pub struct T3Connection {
    uri: T3Uri,
    api_key: String,
    transaction_service_client: TransactionServiceClient,
    logger: Logger,
}

impl T3Connection {
    pub fn new(uri: &T3Uri, api_key: String, logger: Logger) -> Self {
        let env = Arc::new(EnvBuilder::new().name_prefix("T3RPC").build());
        let ch = ChannelBuilder::new(env)
            .max_receive_message_len(std::i32::MAX)
            .max_send_message_len(std::i32::MAX)
            .connect_to_uri(uri, &logger);

        let transaction_service_client = TransactionServiceClient::new(ch);

        Self {
            uri: uri.clone(),
            api_key,
            transaction_service_client,
            logger,
        }
    }

    pub fn find_transactions(&self) -> Result<(), Error> {
        let mut request = FindTransactionsRequest::new();

        let response = self
            .transaction_service_client
            .find_transactions_opt(&request, common_headers_call_option(&self.api_key))
            .map_err(|err| {
                log::warn!(
                    self.logger,
                    "t3_transaction_service find_transactions RPC call failed: {}",
                    err
                );
                err
            })?;

        Ok(())
    }

    pub fn list_transactions() {}

    pub fn create_transaction(
        &self,
        transparent_transaction: TransparentTransaction,
    ) -> Result<CreateTransactionResponse, Error> {
        let mut request = CreateTransactionRequest::new();
        request.set_transaction(transparent_transaction);

        self.transaction_service_client
            .create_transaction_opt(&request, common_headers_call_option(&self.api_key))
            .map_err(|err| {
                log::warn!(
                    self.logger,
                    "t3_transaction_service find_transactions RPC call failed: {}",
                    err
                );
                err
            })
    }

    pub fn test_error(&self) -> Result<(), Error> {
        let mut request = TestErrorRequest::new();
        request.set_code(400);

        let response = self
            .transaction_service_client
            .test_error_opt(&request, common_headers_call_option(&self.api_key))
            .map_err(|err| {
                log::warn!(
                    self.logger,
                    "t3_transaction_service find_transactions RPC call failed: {}",
                    err
                );
                err
            })?;

        Ok(())
    }

    // pub fn get_archive_blocks(&self, offset: u64, limit: u32) ->
    // Result<Vec<ArchiveBlock>, Error> {     let mut request =
    // BlocksRequest::new();     request.set_offset(offset);
    //     request.set_limit(limit);

    //     let response = self
    //         .validator_api_client
    //         .get_archive_blocks_opt(&request,
    // common_headers_call_option(&self.chain_id))         .map_err(|err| {
    //             log::warn!(
    //                 self.logger,
    //                 "validator get_archive_blocks RPC call failed: {}",
    //                 err
    //             );
    //             err
    //         })?;

    //     Ok(response.get_blocks().to_vec())
    // }

    // pub fn get_blocks_data(&self, offset: u64, limit: u32) ->
    // Result<Vec<BlockData>, Error> {     let archive_blocks =
    // self.get_archive_blocks(offset, limit)?;

    //     let blocks_data = archive_blocks
    //         .iter()
    //         .map(BlockData::try_from)
    //         .collect::<Result<Vec<_>, _>>()?;
    //     Ok(blocks_data)
    // }

    // /// Given a fog report uri, fetch its response over grpc, or return an
    // /// error.
    // pub fn fetch_fog_report(&self, uri: &FogUri) -> Result<ReportResponse, Error>
    // {     let mut request = FetchFogReportRequest::new();
    //     request.set_uri(uri.to_string());

    //     let response = self
    //         .validator_api_client
    //         .fetch_fog_report_opt(&request,
    // common_headers_call_option(&self.chain_id))         .map_err(|err| {
    //             log::warn!(
    //                 self.logger,
    //                 "validator fetch_fog_report RPC call failed: {}",
    //                 err
    //             );
    //             err
    //         })?;

    //     match response.get_result() {
    //         FetchFogReportResult::Ok => Ok(response.get_report().clone()),

    //         FetchFogReportResult::NoReports => Err(Error::NoReports),
    //     }
    // }

    // /// Fetch multiple fog reports.
    // pub fn fetch_fog_reports(
    //     &self,
    //     uris: impl Iterator<Item = FogUri>,
    // ) -> Result<FogReportResponses, Error> {
    //     let mut responses = FogReportResponses::default();
    //     for uri in uris {
    //         if responses.contains_key(&uri.to_string()) {
    //             continue;
    //         }

    //         let response = self.fetch_fog_report(&uri)?;
    //         responses.insert(uri.to_string(), response.into());
    //     }

    //     Ok(responses)
    // }
}

impl Display for T3Connection {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.uri)
    }
}

impl Eq for T3Connection {}

impl Hash for T3Connection {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.uri.addr().hash(hasher);
    }
}

impl PartialEq for T3Connection {
    fn eq(&self, other: &Self) -> bool {
        self.uri.addr() == other.uri.addr()
    }
}

impl Ord for T3Connection {
    fn cmp(&self, other: &Self) -> Ordering {
        self.uri.addr().cmp(&other.uri.addr())
    }
}

impl PartialOrd for T3Connection {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.uri.addr().partial_cmp(&other.uri.addr())
    }
}

impl Connection for T3Connection {
    type Uri = T3Uri;

    fn uri(&self) -> Self::Uri {
        self.uri.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
