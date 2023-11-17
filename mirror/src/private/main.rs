// Copyright (c) 2018-2021 MobileCoin Inc.

//! The private side of wallet-service-mirror.
//! This program forms outgoing connections to both a wallet service instance,
//! as well as a public wallet-service-mirror instance. It then proceeds to poll
//! the public side of the mirror for requests which it then forwards to the
//! wallet service. When a response is received it is then forwarded back to the
//! mirror.

mod crypto;

use crate::crypto::{decrypt, encrypt, load_private_key};
use boring::{pkey::Private, rsa::Rsa};
use grpcio::ChannelBuilder;
use mc_common::logger::{create_app_logger, log, o, Logger};
use mc_full_service_mirror::{
    uri::WalletServiceMirrorUri,
    wallet_service_mirror_api::{
        EncryptedResponse, PollRequest, QueryRequest, QueryResponse, UnencryptedResponse,
    },
    wallet_service_mirror_api_grpc::WalletServiceMirrorClient,
};
use mc_util_grpc::ConnectionUriGrpcioChannel;
use std::{collections::HashMap, str::FromStr, sync::Arc, thread::sleep, time::Duration};
use structopt::StructOpt;

const SUPPORTED_ENDPOINTS: &[&str] = &[
    "check_receiver_receipt_status",
    "create_payment_request",
    "get_account",
    "get_account_status",
    "get_address_for_account",
    "get_addresses_for_account",
    "get_address_status",
    "get_accounts",
    "get_transaction_logs",
    "get_block",
    "get_confirmations",
    "get_network_status",
    "get_transaction_log",
    "get_wallet_status",
    "validate_confirmation",
    "validate_sender_memo",
    "verify_address",
    "get_txos",
    "get_all_accounts",
    "get_all_transaction_logs_for_block",
    "get_balance_for_account",
    "get_balance_for_address",
    "get_transaction_logs_for_account",
];

/// How long do we wait for full-service to reply?
const FULL_SERVICE_TIMEOUT: Duration = Duration::from_secs(120);

/// A wrapper to ease monitor id parsing from a hex string when using
/// `StructOpt`.
#[derive(Clone, Debug)]
pub struct MonitorId(pub Vec<u8>);
impl FromStr for MonitorId {
    type Err = String;
    fn from_str(src: &str) -> Result<Self, Self::Err> {
        let bytes =
            hex::decode(src).map_err(|err| format!("Error decoding monitor id: {err:?}"))?;
        if bytes.len() != 32 {
            return Err("monitor id needs to be exactly 32 bytes".into());
        }
        Ok(Self(bytes))
    }
}

/// Command line config
#[derive(Clone, Debug, StructOpt)]
#[structopt(
    name = "wallet-service-mirror-private",
    about = "The private side of wallet-service-mirror, receiving requests from the public side and forwarding them to the wallet service."
)]
pub struct Config {
    /// Wallet service URI.
    #[structopt(long, default_value = "http://127.0.0.1:9090/")]
    pub wallet_service_uri: String,

    /// URI for the public side of the mirror.
    #[structopt(long)]
    pub mirror_public_uri: WalletServiceMirrorUri,

    /// How many milliseconds to wait between polling.
    #[structopt(long, default_value = "100", parse(try_from_str=parse_duration_in_milliseconds))]
    pub poll_interval: Duration,

    /// Optional encryption public key. If provided, only encrypted requests are
    /// accepted. See `example-client.js` for an example on how to submit
    /// encrypted requests through the mirror.
    #[structopt(long, parse(try_from_str=load_private_key))]
    pub mirror_key: Option<Rsa<Private>>,
}

fn main() {
    mc_common::setup_panic_handler();
    let _sentry_guard = mc_common::sentry::init();

    let config = Config::from_args();

    let (logger, _global_logger_guard) = create_app_logger(o!());
    log::info!(
        logger,
        "Starting wallet-service-mirror private forwarder on {}, connecting to wallet service at {}",
        config.mirror_public_uri,
        config.wallet_service_uri,
    );

    // Set up the gRPC connection to the public side of the mirror.
    let mirror_api_client = {
        let env = Arc::new(grpcio::EnvBuilder::new().build());
        let ch = ChannelBuilder::new(env)
            .max_receive_message_len(-1)
            .max_send_message_len(-1)
            .max_reconnect_backoff(Duration::from_millis(2000))
            .initial_reconnect_backoff(Duration::from_millis(1000))
            .connect_to_uri(&config.mirror_public_uri, &logger);

        WalletServiceMirrorClient::new(ch)
    };

    // Main polling loop.
    log::debug!(logger, "Entering main loop");

    let mut pending_responses: HashMap<String, QueryResponse> = HashMap::new();

    loop {
        // Communicate with the public side of the mirror.
        let mut request = PollRequest::new();
        request.set_query_responses(pending_responses.clone());

        log::debug!(
            logger,
            "Calling poll with {} queued responses",
            pending_responses.len()
        );
        match mirror_api_client.poll(&request) {
            Ok(response) => {
                log::debug!(
                    logger,
                    "Poll succeeded, got back {} requests",
                    response.query_requests.len()
                );

                // Clear pending responses since we successfully delivered them to the other
                // side.
                pending_responses.clear();

                // Process requests.
                for (query_id, query_request) in response.query_requests.iter() {
                    let query_logger = logger.new(o!("query_id" => query_id.clone()));

                    let response = {
                        if let Some(mirror_key) = config.mirror_key.as_ref() {
                            process_encrypted_request(
                                &config.wallet_service_uri,
                                mirror_key,
                                query_request,
                                &query_logger,
                            )
                            .unwrap_or_else(|err| {
                                log::error!(
                                    query_logger,
                                    "process_encrypted_request failed: {:?}",
                                    err
                                );

                                let mut err_query_response = QueryResponse::new();
                                err_query_response.set_error(err);
                                err_query_response
                            })
                        } else {
                            process_unencrypted_request(
                                &config.wallet_service_uri,
                                query_request,
                                &query_logger,
                            )
                            .unwrap_or_else(|err| {
                                log::error!(
                                    query_logger,
                                    "process_unencrypted_request failed: {:?}",
                                    err
                                );

                                let mut err_query_response = QueryResponse::new();
                                err_query_response.set_error(err);
                                err_query_response
                            })
                        }
                    };

                    pending_responses.insert(query_id.clone(), response);
                }
            }

            Err(err) => {
                log::error!(
                    logger,
                    "Polling the public side of the mirror failed: {:?}",
                    err
                );
            }
        }

        sleep(config.poll_interval);
    }
}

fn validate_method(json: &str) -> serde_json::Result<bool> {
    let json: serde_json::Value = serde_json::from_str(json)?;
    let method = json["method"].as_str().unwrap_or("");
    Ok(SUPPORTED_ENDPOINTS.iter().any(|&s| s == method))
}

fn process_unencrypted_request(
    wallet_service_uri: &str,
    query_request: &QueryRequest,
    logger: &Logger,
) -> Result<QueryResponse, String> {
    if !query_request.has_unencrypted_request() {
        return Err("Only processing unencrypted requests".into());
    }

    let unencrypted_request = query_request.get_unencrypted_request();

    log::debug!(
        logger,
        "Incoming unencrypted request ({})",
        unencrypted_request.json_request
    );

    // Check that the request is of an allowed type.
    match validate_method(&unencrypted_request.json_request) {
        Ok(true) => (),
        Ok(false) => return Err("Unsupported request".into()),
        Err(err) => {
            let mut err_query_response = QueryResponse::new();
            err_query_response.set_error(format!("Error parsing JSON request: {err}"));
            return Ok(err_query_response);
        }
    }

    // Pass request along to full-service.
    let client = reqwest::blocking::Client::builder()
        .timeout(FULL_SERVICE_TIMEOUT)
        .build()
        .map_err(|e| e.to_string())?;
    let res = client
        .post(wallet_service_uri)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(unencrypted_request.json_request.clone())
        .send()
        .map_err(|e| e.to_string())?;
    let json_response = res.text().map_err(|e| e.to_string())?;

    let mut unencrypted_response = UnencryptedResponse::new();
    unencrypted_response.set_json_response(json_response);

    let mut mirror_response = QueryResponse::new();
    mirror_response.set_unencrypted_response(unencrypted_response);
    Ok(mirror_response)
}

fn process_encrypted_request(
    wallet_service_uri: &str,
    mirror_key: &Rsa<Private>,
    query_request: &QueryRequest,
    logger: &Logger,
) -> Result<QueryResponse, String> {
    if !query_request.has_encrypted_request() {
        return Err("Only processing encrypted requests".into());
    }

    let encrypted_request = query_request.get_encrypted_request();

    // Decrypt the request.
    let json_request = match decrypt(mirror_key, &encrypted_request.payload)
        .map_err(|err| format!("Error decrypting request: {err}"))
        .and_then(|decrypted| {
            String::from_utf8(decrypted).map_err(|err| format!("Error parsing utf8: {err}"))
        }) {
        Ok(json_request) => json_request,
        Err(err) => {
            let mut err_query_response = QueryResponse::new();
            err_query_response.set_error(err);
            return Ok(err_query_response);
        }
    };

    log::debug!(logger, "Incoming encrypted request ({})", json_request,);

    // Check that the request is of an allowed type.
    match validate_method(&json_request) {
        Ok(true) => (),
        Ok(false) => return Err("Unsupported request".into()),
        Err(err) => {
            let mut err_query_response = QueryResponse::new();
            err_query_response.set_error(format!("Error parsing JSON request: {err}"));
            return Ok(err_query_response);
        }
    }

    // Pass request along to full-service.
    let client = reqwest::blocking::Client::builder()
        .timeout(FULL_SERVICE_TIMEOUT)
        .build()
        .map_err(|e| e.to_string())?;
    let res = client
        .post(wallet_service_uri)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(json_request)
        .send()
        .map_err(|e| e.to_string())?;
    let json_response = res.text().map_err(|e| e.to_string())?;

    let encrypted_payload =
        encrypt(mirror_key, json_response.as_bytes()).map_err(|_e| "Encryption failed")?;

    let mut encrypted_response = EncryptedResponse::new();
    encrypted_response.set_payload(encrypted_payload);

    let mut mirror_response = QueryResponse::new();
    mirror_response.set_encrypted_response(encrypted_response);
    Ok(mirror_response)
}

fn parse_duration_in_milliseconds(src: &str) -> Result<Duration, std::num::ParseIntError> {
    Ok(Duration::from_millis(u64::from_str(src)?))
}
