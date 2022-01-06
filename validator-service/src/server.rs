use futures::executor::block_on;
use grpcio::{Server as GrpcioServer, ServerBuilder};
use mc_common::logger::{log, Logger};
use mc_ledger_sync::LedgerSyncServiceThread;
use std::sync::Arc;

const NETWORK: &str = "test";

/// The application server
pub struct Server {
    server: GrpcioServer,
    ledger_sync: Option<LedgerSyncServiceThread>,
    logger: Logger,
}

impl Server {
    pub fn new(logger: Logger) -> Self {
        //TODO update name
        log::info!(logger, "starting, network = {}", NETWORK);

        let env = Arc::new(
            grpcio::EnvBuilder::new()
                .name_prefix("ValidatorNode-RPC".to_string())
                .build(),
        );

        let server_builder = ServerBuilder::new(env);
        log::info!(logger, "Registered service");

        let server = server_builder.build().unwrap();
        Self {
            server,
            ledger_sync: None,
            logger,
        }
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
        match &mut self.ledger_sync {
            // The division was valid
            Some(ledger_sync) => {
                ledger_sync.stop();
            }
            // The division was invalid
            None => println!("Cannot divide by 0"),
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.stop();
    }
}
