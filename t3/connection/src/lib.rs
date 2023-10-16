mod error;

use mc_common::logger::{log, Logger};
use mc_connection::Connection;
use mc_util_grpc::ConnectionUriGrpcioChannel;
use mc_util_uri::ConnectionUri;
use t3_api::{
    t3_grpc::TransactionServiceClient, CreateTransactionRequest, CreateTransactionResponse,
    FindTransactionsRequest, FindTransactionsResponse, T3Uri, TestErrorRequest, TestErrorResponse,
    TransparentTransaction,
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

    pub fn find_transactions(&self) -> Result<FindTransactionsResponse, Error> {
        let request = FindTransactionsRequest::new();

        Ok(self
            .transaction_service_client
            .find_transactions_opt(&request, common_headers_call_option(&self.api_key))
            .map_err(|err| {
                log::warn!(
                    self.logger,
                    "t3_transaction_service find_transactions RPC call failed: {}",
                    err
                );
                err
            })?)
    }

    pub fn list_transactions() {}

    pub fn create_transaction(
        &self,
        transparent_transaction: TransparentTransaction,
    ) -> Result<CreateTransactionResponse, Error> {
        let mut request = CreateTransactionRequest::new();
        request.set_transaction(transparent_transaction);

        Ok(self
            .transaction_service_client
            .create_transaction_opt(&request, common_headers_call_option(&self.api_key))
            .map_err(|err| {
                log::warn!(
                    self.logger,
                    "t3_transaction_service find_transactions RPC call failed: {}",
                    err
                );
                err
            })?)
    }

    pub fn test_error(&self, code: i32) -> Result<TestErrorResponse, Error> {
        let mut request = TestErrorRequest::new();
        request.set_code(code);

        Ok(self
            .transaction_service_client
            .test_error_opt(&request, common_headers_call_option(&self.api_key))
            .map_err(|err| {
                log::warn!(
                    self.logger,
                    "t3_transaction_service find_transactions RPC call failed: {}",
                    err
                );
                err
            })?)
    }
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
mod tests {}
