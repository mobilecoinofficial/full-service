use crate::validator_blockchain_service::BlockchainApiService;
use futures::executor::block_on;
use grpcio::{Server as GrpcioServer, ServerBuilder};
use mc_common::logger::{log, Logger};
use mc_consensus_api::consensus_common_grpc;
use mc_ledger_db::LedgerDB;
use mc_util_grpc::{AnonymousAuthenticator, Authenticator};
use std::sync::Arc;

const NETWORK: &str = "test";

/// The application server
pub struct Server {
    server: GrpcioServer,
    logger: Logger,
}

impl Server {
    pub fn new(ledger_db: LedgerDB, logger: Logger) -> Self {
        //TODO update name
        log::info!(logger, "starting, network = {}", NETWORK);

        let env = Arc::new(
            grpcio::EnvBuilder::new()
                .name_prefix("ValidatorNode-RPC".to_string())
                .build(),
        );
        // Authenticator
        let client_authenticator: Arc<dyn Authenticator + Sync + Send> =
            Arc::new(AnonymousAuthenticator::default());
        let blockchain_service =
            consensus_common_grpc::create_blockchain_api(BlockchainApiService::new(
                ledger_db,
                client_authenticator,
                logger.clone(),
                Some(0),
            ));

        let server_builder = ServerBuilder::new(env).register_service(blockchain_service);
        log::info!(logger, "Registered service");

        let server = server_builder.build().unwrap();
        Self { server, logger }
    }

    pub fn start(&mut self) {
        self.server.start();
        log::info!(
            self.logger,
            "Inside Server Start: Binding to {} host/ports",
            self.server.bind_addrs().len()
        );
        for (host, port) in self.server.bind_addrs() {
            log::info!(self.logger, "API listening on {}:{}", host, port);
        }
    }

    pub fn stop(&mut self) {
        block_on(self.server.shutdown()).expect("Could not stop grpc server");
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.stop();
    }
}
