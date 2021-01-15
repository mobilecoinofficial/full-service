// Copyright (c) 2018-2020 MobileCoin Inc.

//! The public side of wallet-service-mirror.
//! This program opens two listening ports:
//! 1) A GRPC server for receiving incoming poll requests from the private side of the mirror
//! 2) An http(s) server for receiving client requests which will then be forwarded to the
//!    full service instance sitting behind the private part of the mirror.

#![feature(decl_macro)]

mod mirror_service;
mod query;
mod utils;

use grpcio::{EnvBuilder, ServerBuilder};
use mc_common::logger::{create_app_logger, log, o, Logger};
use mc_util_grpc::{BuildInfoService, ConnectionUriGrpcioServer, HealthService};
use mc_util_uri::{ConnectionUri, Uri, UriScheme};
use mc_wallet_service_mirror::{
    uri::WalletServiceMirrorUri,
    wallet_mirror_api::{QueryRequest, SignedRequest},
};
use mirror_service::MirrorService;
use query::QueryManager;
use rocket::{
    config::{Config as RocketConfig, Environment as RocketEnvironment},
    http::Status,
    post,
    response::Responder,
    routes, Request, Response,
};
use rocket_contrib::json::Json;
use serde::Deserialize;
use std::sync::Arc;
use structopt::StructOpt;

pub type ClientUri = Uri<ClientUriScheme>;

/// Wallet Service Mirror Uri Scheme
#[derive(Debug, Hash, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct ClientUriScheme {}
impl UriScheme for ClientUriScheme {
    /// The part before the '://' of a URL.
    const SCHEME_SECURE: &'static str = "https";
    const SCHEME_INSECURE: &'static str = "http";

    /// Default port numbers
    const DEFAULT_SECURE_PORT: u16 = 8443;
    const DEFAULT_INSECURE_PORT: u16 = 8000;
}

/// Command line config
#[derive(Clone, Debug, StructOpt)]
#[structopt(
    name = "wallet-service-mirror-public",
    about = "The public side of wallet-service-mirror, receiving requests from clients and forwarding them to wallet servic through the private side of the mirror"
)]
pub struct Config {
    /// Listening URI for the private-public interface connection (GRPC).
    #[structopt(long)]
    pub mirror_listen_uri: WalletServiceMirrorUri,

    /// Listening URI for client requests (HTTP(S)).
    #[structopt(long)]
    pub client_listen_uri: ClientUri,

    /// Override the number of workers used for the client http server.
    /// This controls how many concurrent requests the server can process.
    #[structopt(long)]
    pub num_workers: Option<u16>,

    /// Allow using self-signed TLS certificate for GRPC connections.
    #[structopt(long)]
    pub allow_self_signed_tls: bool,
}

/// State that is accessible by all rocket requests
struct State {
    query_manager: QueryManager,
    logger: Logger,
}

/// Sets the status of the response to 400 (Bad Request).
#[derive(Debug, Clone, PartialEq)]
pub struct BadRequest(pub String);

/// Sets the status code of the response to 400 Bad Request and include an error message in the
/// response.
impl<'r> Responder<'r> for BadRequest {
    fn respond_to(self, req: &Request) -> Result<Response<'r>, Status> {
        let mut build = Response::build();
        build.merge(self.0.respond_to(req)?);

        build.status(Status::BadRequest).ok()
    }
}
impl From<&str> for BadRequest {
    fn from(src: &str) -> Self {
        Self(src.to_owned())
    }
}
impl From<String> for BadRequest {
    fn from(src: String) -> Self {
        Self(src)
    }
}

#[derive(Deserialize)]
struct JsonSignedRequest {
    request: String,
    signature: Vec<u8>,
}

#[post("/signed-request", format = "json", data = "<request>")]
fn signed_request(
    state: rocket::State<State>,
    request: Json<JsonSignedRequest>,
) -> Result<Vec<u8>, BadRequest> {
    let mut signed_request = SignedRequest::new();
    signed_request.set_json_request(request.request.clone());
    signed_request.set_signature(request.signature.clone());

    let mut query_request = QueryRequest::new();
    query_request.set_signed_request(signed_request);

    log::debug!(
        state.logger,
        "Enqueueing SignedRequest({})",
        request.request,
    );
    let query = state.query_manager.enqueue_query(query_request);
    let query_response = query.wait()?;

    if query_response.has_error() {
        log::error!(
            state.logger,
            "SignedRequest({}) failed: {}",
            request.request,
            query_response.get_error()
        );
        return Err(query_response.get_error().into());
    }
    if !query_response.has_encrypted_response() {
        log::error!(
            state.logger,
            "SignedRequest({}) returned incorrect response type",
            request.request,
        );
        return Err("Incorrect response type received".into());
    }

    log::info!(
        state.logger,
        "SignedRequest({}) completed successfully",
        request.request,
    );

    let response = query_response.get_encrypted_response();
    Ok(response.get_payload().to_vec())
}

fn main() {
    mc_common::setup_panic_handler();
    let _sentry_guard = mc_common::sentry::init();

    let config = Config::from_args();
    if !config.allow_self_signed_tls
        && utils::is_tls_self_signed(&config.mirror_listen_uri).expect("is_tls_self_signed failed")
    {
        panic!("Refusing to start with self-signed TLS certificate. Use --allow-self-signed-tls to override this check.");
    }

    let (logger, _global_logger_guard) = create_app_logger(o!());
    log::info!(
        logger,
        "Starting wallet service mirror public forwarder, listening for mirror requests on {} and client requests on {}",
        config.mirror_listen_uri.addr(),
        config.client_listen_uri.addr(),
    );

    // Common state.
    let query_manager = QueryManager::default();

    // Start the mirror-facing GRPC server.
    log::info!(logger, "Starting mirror GRPC server");

    let build_info_service = BuildInfoService::new(logger.clone()).into_service();
    let health_service = HealthService::new(None, logger.clone()).into_service();
    let mirror_service = MirrorService::new(query_manager.clone(), logger.clone()).into_service();

    let env = Arc::new(
        EnvBuilder::new()
            .name_prefix("Mirror-RPC".to_string())
            .build(),
    );

    let server_builder = ServerBuilder::new(env)
        .register_service(build_info_service)
        .register_service(health_service)
        .register_service(mirror_service)
        .bind_using_uri(&config.mirror_listen_uri);

    let mut server = server_builder.build().unwrap();
    server.start();

    // Start the client-facing webserver.
    if config.client_listen_uri.use_tls() {
        panic!("Client-listening using TLS is currently not supported due to `ring` crate version compatibility issues.");
    }

    let mut rocket_config = RocketConfig::build(
        RocketEnvironment::active().expect("Failed getitng rocket environment"),
    )
    .address(config.client_listen_uri.host())
    .port(config.client_listen_uri.port());
    if config.client_listen_uri.use_tls() {
        rocket_config = rocket_config.tls(
            config
                .client_listen_uri
                .tls_chain_path()
                .expect("failed getting tls chain path"),
            config
                .client_listen_uri
                .tls_key_path()
                .expect("failed getting tls key path"),
        );
    }
    if let Some(num_workers) = config.num_workers {
        rocket_config = rocket_config.workers(num_workers);
    }
    let rocket_config = rocket_config
        .finalize()
        .expect("Failed creating client http server config");

    log::info!(logger, "Starting client web server");
    rocket::custom(rocket_config)
        .mount("/", routes![signed_request,])
        .manage(State {
            query_manager,
            logger,
        })
        .launch();
}
