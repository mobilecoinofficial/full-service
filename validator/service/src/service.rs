// Copyright 2018-2022 MobileCoin, Inc.

//! Ledger Validator Node GRPC Service.

use crate::{blockchain_api::BlockchainApi, validator_api::ValidatorApi};
use grpcio::{EnvBuilder, ServerBuilder};
use mc_common::logger::{log, Logger};
use mc_connection::{BlockchainConnection, ConnectionManager, UserTxConnection};
use mc_ledger_db::LedgerDB;
use mc_util_grpc::{BuildInfoService, ConnectionUriGrpcioServer, HealthService};
use mc_validator_api::ValidatorUri;
use std::sync::Arc;

pub struct Service {
    /// GRPC server.
    _server: grpcio::Server,
}

impl Service {
    pub fn new<C: BlockchainConnection + UserTxConnection + 'static>(
        listen_uri: &ValidatorUri,
        chain_id: String,
        ledger_db: LedgerDB,
        conn_manager: ConnectionManager<C>,
        logger: Logger,
    ) -> Self {
        // Build info API service.
        let build_info_service = BuildInfoService::new(logger.clone()).into_service();

        // Health check service.
        let health_service = HealthService::new(None, logger.clone()).into_service();

        // Validator API service.
        let validator_service = ValidatorApi::new(
            chain_id,
            ledger_db.clone(),
            conn_manager.clone(),
            logger.clone(),
        )
        .into_service();

        // Blockchain API service.
        let blockchain_service =
            BlockchainApi::new(ledger_db, conn_manager, logger.clone()).into_service();

        // Package service into grpc server.
        log::info!(logger, "Starting validator API Service on {}", listen_uri);
        let env = Arc::new(
            EnvBuilder::new()
                .name_prefix("Validator-RPC".to_string())
                .build(),
        );

        let server = ServerBuilder::new(env)
            .register_service(build_info_service)
            .register_service(health_service)
            .register_service(validator_service)
            .register_service(blockchain_service)
            .build_using_uri(uri, logger)
            .expect("Failed to build grpc server");

        server.start();

        Self { _server: server }
    }
}
