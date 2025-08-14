// Copyright (c) 2018-2022 MobileCoin Inc.

//! The public side of wallet-service-mirror.
//! This program opens two listening ports:
//! 1) A GRPC server for receiving incoming poll requests from the private side
//!    of the mirror
//! 2) An http(s) server for receiving client requests which will then be
//!    forwarded to the wallet service instance sitting behind the private part
//!    of the mirror.

#![feature(decl_macro)]

mod mirror_service;
mod query;
mod utils;

use mirror_service::MirrorService;

use mc_common::logger::{create_app_logger, log, o, Logger};
use mc_full_service_mirror::{
    uri::WalletServiceMirrorUri,
    wallet_service_mirror_api::{EncryptedRequest, QueryRequest, UnencryptedRequest},
};
use mc_util_grpc::{BuildInfoService, ConnectionUriGrpcioServer, HealthService};
use mc_util_uri::{ConnectionUri, Uri, UriScheme};

use grpcio::{ChannelBuilder, EnvBuilder, ServerBuilder};
use query::QueryManager;
use rocket::{
    config::{Config as RocketConfig, TlsConfig},
    data::ToByteUnit,
    http::Status,
    post,
    response::Responder,
    routes,
    tokio::io::AsyncReadExt,
    Build, Data, Request, Response, Rocket,
};
use structopt::StructOpt;

use std::{net::IpAddr, str::FromStr, sync::Arc};

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
    about = "The public side of wallet-service-mirror, receiving requests from clients and forwarding them to the wallet service through the private side of the mirror"
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

/// Sets the status code of the response to 400 Bad Request and include an error
/// message in the response.
impl<'r> Responder<'r, 'static> for BadRequest {
    fn respond_to(self, req: &'r Request<'_>) -> Result<Response<'static>, Status> {
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

#[post("/unencrypted-request", format = "json", data = "<request_data>")]
async fn unencrypted_request(
    state: &rocket::State<State>,
    request_data: rocket::Data<'_>,
) -> Result<String, BadRequest> {
    let mut request = String::new();
    let res = request_data
        .open(2.mebibytes())
        .read_to_string(&mut request)
        .await;
    if res.is_err() {
        let msg = "Could not read request data for unencrypted request.";
        log::error!(state.logger, "{}", msg,);
        return Err(msg.into());
    }

    log::debug!(state.logger, "Enqueueing UnencryptedRequest({})", &request);

    let mut unencrypted_request = UnencryptedRequest::new();
    unencrypted_request.set_json_request(request.clone());

    let mut query_request = QueryRequest::new();
    query_request.set_unencrypted_request(unencrypted_request);

    let query = state.query_manager.enqueue_query(query_request);
    let query_response = query.wait()?;

    if query_response.has_error() {
        log::error!(
            state.logger,
            "UnencryptedRequest({}) failed: {}",
            request,
            query_response.get_error()
        );
        return Err(query_response.get_error().into());
    }
    if !query_response.has_unencrypted_response() {
        log::error!(
            state.logger,
            "UnencryptedRequest({}) returned incorrect response type",
            request,
        );
        return Err("Incorrect response type received".into());
    }

    log::info!(
        state.logger,
        "UnencryptedRequest({}) completed successfully",
        request,
    );

    let response = query_response.get_unencrypted_response();
    Ok(response.get_json_response().to_string())
}

#[post(
    "/encrypted-request",
    format = "application/octet-stream",
    data = "<data>"
)]
async fn encrypted_request(
    state: &rocket::State<State>,
    data: Data<'_>,
) -> Result<Vec<u8>, BadRequest> {
    let mut payload = Vec::new();
    if let Err(err) = data.open(2.mebibytes()).read_to_end(&mut payload).await {
        let msg = format!("Could not read request data for unencrypted request: {err}");
        log::error!(state.logger, "{}", msg);
        return Err(msg.into());
    }
    let payload_len = payload.len();

    let mut encrypted_request = EncryptedRequest::new();
    encrypted_request.set_payload(payload);

    let mut query_request = QueryRequest::new();
    query_request.set_encrypted_request(encrypted_request);

    log::debug!(
        state.logger,
        "Enqueueing EncryptedRequest({} bytes)",
        payload_len,
    );
    let query = state.query_manager.enqueue_query(query_request);
    let query_response = query.wait()?;

    if query_response.has_error() {
        log::error!(
            state.logger,
            "EncryptedRequest({} bytes) failed: {}",
            payload_len,
            query_response.get_error()
        );
        return Err(query_response.get_error().into());
    }
    if !query_response.has_encrypted_response() {
        log::error!(
            state.logger,
            "EncryptedRequest({} bytes) returned incorrect response type",
            payload_len,
        );
        return Err("Incorrect response type received".into());
    }

    log::info!(
        state.logger,
        "EncryptedRequest({} bytes) completed successfully",
        payload_len,
    );

    let response = query_response.get_encrypted_response();
    Ok(response.get_payload().to_vec())
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    mc_common::setup_panic_handler();
    let _sentry_guard = mc_common::sentry::init();

    let config = Config::from_args();

    let (logger, global_logger_guard) = create_app_logger(o!());

    // This function is wrapped inside rocket::main which creates a tokio runtime
    // and then calls us. Some random crates log stuff after this function
    // returns as part of the shutdown process, and that results in a panic
    // since there is no global logger (they use the `log` crate which calls
    // `slog-scope`). The workaround we have here is to instruct `slog-scope` to not
    // unset the global logger when returning. It will keep holding a reference
    // to the `Logger` object, so the fact that we drop our own reference is not
    // going to be an issue.
    global_logger_guard.cancel_reset();

    let query_manager = QueryManager::default();

    log::info!(
        logger.clone(),
        "Starting wallet service mirror public forwarder, listening for mirror requests on {} and client requests on {}",
        config.mirror_listen_uri.addr(),
        config.client_listen_uri.addr(),
    );

    // Start the mirror-facing GRPC server.
    log::info!(logger.clone(), "Starting mirror GRPC server");

    let mut server = build_grpc_server(&config, &query_manager, &logger);
    server.start();

    let rocket = build_rocket(config, query_manager, logger);

    let _ = rocket.launch().await?;

    Ok(())
}

/// Starts the GRPC server in its own thread. This is necessary because the GRPC
/// server does not implement Sync, which is a requirement to be managed by
/// Rocket OR pass over an async await block.
fn build_grpc_server(
    config: &Config,
    query_manager: &QueryManager,
    logger: &Logger,
) -> grpcio::Server {
    let build_info_service = BuildInfoService::new(logger.clone()).into_service();
    let health_service = HealthService::new(None, logger.clone()).into_service();
    let mirror_service = MirrorService::new(query_manager.clone(), logger.clone()).into_service();

    let env = Arc::new(
        EnvBuilder::new()
            .name_prefix("Mirror-RPC".to_string())
            .build(),
    );

    let ch_builder = ChannelBuilder::new(env.clone())
        .max_receive_message_len(-1)
        .max_send_message_len(-1);

    ServerBuilder::new(env)
        .register_service(build_info_service)
        .register_service(health_service)
        .register_service(mirror_service)
        .channel_args(ch_builder.build_args())
        .build_using_uri(&config.mirror_listen_uri, logger.clone())
        .expect("Failed to build mirror GRPC server")
}

/// Builds the rocket instance given the configuration.
fn build_rocket(config: Config, query_manager: QueryManager, logger: Logger) -> Rocket<Build> {
    // Start the client-facing webserver.
    if config.client_listen_uri.use_tls() {
        panic!("Client-listening using TLS is currently not supported due to `ring` crate version compatibility issues.");
    }

    let tls_config = if config.client_listen_uri.use_tls() {
        Some(TlsConfig::from_paths(
            config
                .client_listen_uri
                .tls_chain_path()
                .expect("failed getting tls chain path"),
            config
                .client_listen_uri
                .tls_key_path()
                .expect("failed getting tls key path"),
        ))
    } else {
        None
    };

    let rocket_config = RocketConfig {
        address: IpAddr::from_str(&config.client_listen_uri.host()).expect("failed parsing host"),
        port: config.client_listen_uri.port(),
        tls: tls_config,
        workers: config
            .num_workers
            .map(|n| n as usize)
            .unwrap_or_else(num_cpus::get),
        ..RocketConfig::default()
    };

    log::info!(logger, "Starting client web server");
    rocket::custom(rocket_config)
        .manage(State {
            query_manager,
            logger,
        })
        .mount("/", routes![unencrypted_request, encrypted_request])
}
