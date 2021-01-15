// Copyright (c) 2018-2020 MobileCoin Inc.

mod crypto;
mod request;

use grpcio::{ChannelBuilder, RpcStatus, RpcStatusCode};
use mc_common::logger::{create_app_logger, log, o, Logger};
use mc_wallet_service_mirror::{
    wallet_mirror_api_grpc::WalletServiceMirrorClient,
    uri::WalletServiceMirrorUri,
    wallet_mirror_api::{EncryptedResponse, PollRequest, QueryRequest, QueryResponse},
};
use mc_util_grpc::ConnectionUriGrpcioChannel;
use reqwest;
use rsa::RSAPublicKey;
use serde_json::Value;
use std::{
    collections::HashMap, convert::TryFrom, str::FromStr, sync::Arc, thread::sleep, time::Duration,
};
use structopt::StructOpt;

/// Command line config
#[derive(Clone, Debug, StructOpt)]
#[structopt(
    name = "wallet-service-mirror-private",
    about = "The private side of wallet-service-mirror, receiving requests from the public side and forwarding them to the wallet service"
)]
pub struct Config {
    /// Wallet service endpoint.
    #[structopt(long, default_value = "http://127.0.0.1:9090/wallet")]
    pub wallet_service_endpoint: String,

    /// URI for the public side of the mirror.
    #[structopt(long)]
    pub mirror_public_uri: WalletServiceMirrorUri,

    /// How many milliseconds to wait between polling.
    #[structopt(long, default_value = "100", parse(try_from_str=parse_duration_in_milliseconds))]
    pub poll_interval: Duration,

    /// AccountId to use by default for operations that require it
    #[structopt(long)]
    pub account_id: String,

    /// Optional encryption public key. 
    /// See `example-client.js` for an example on how to submit encrypted requests
    /// through the mirror.
    #[structopt(long, parse(try_from_str=load_public_key))]
    pub mirror_key: Option<RSAPublicKey>,
}

fn main() {
    mc_common::setup_panic_handler();
    let _sentry_guard = mc_common::sentry::init();

    let config = Config::from_args();

    let (logger, _global_logger_guard) = create_app_logger(o!());
    log::info!(
        logger,
        "Starting wallet service mirror private forwarder on {}, connecting to wallet service {}",
        config.mirror_public_uri,
        config.wallet_service_endpoint,
    );

    // Set up the gRPC connection to the public side of the mirror.
    let mirror_api_client = {
        let env = Arc::new(grpcio::EnvBuilder::new().build());
        let ch = ChannelBuilder::new(env)
            .max_receive_message_len(std::i32::MAX)
            .max_send_message_len(std::i32::MAX)
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

                // Clear pending responses since we successfully delivered them to the other side.
                pending_responses.clear();

                // Process requests.
                for (query_id, query_request) in response.query_requests.iter() {
                    let query_logger = logger.new(o!("query_id" => query_id.clone()));

                    let response = {
                        if let Some(mirror_key) = config.mirror_key.as_ref() {
                            process_encrypted_request(
                                &config.wallet_service_endpoint,
                                &config.account_id,
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
                                err_query_response.set_error(err.to_string());
                                err_query_response
                            })
                        } else {
                            let mut err_query_response = QueryResponse::new();
                            err_query_response.set_error("Unsupported".to_string());
                            err_query_response
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

fn process_encrypted_request(
    wallet_service_uri: &String,
    _account_id: &String,
    mirror_key: &RSAPublicKey,
    query_request: &QueryRequest,
    logger: &Logger,
) -> grpcio::Result<QueryResponse> {
    if !query_request.has_signed_request() {
        return Err(grpcio::Error::RpcFailure(RpcStatus::new(
            RpcStatusCode::INTERNAL,
            Some("Only processing signed requests".into()),
        )));
    }

    let signed_request = query_request.get_signed_request();

    log::debug!(
        logger,
        "Incoming signed request ({})",
        signed_request.json_request
    );
    let sig_is_valid = crypto::verify_sig(
        mirror_key,
        signed_request.json_request.as_bytes(),
        &signed_request.signature,
    )
    .is_ok();

    if !sig_is_valid {
        let mut err_query_response = QueryResponse::new();
        err_query_response.set_error("Signature verification failed".to_owned());
        return Ok(err_query_response);
    }

    let json_request: Value = match serde_json::from_str(&signed_request.json_request) {
        Ok(req) => req,
        Err(err) => {
            let mut err_query_response = QueryResponse::new();
            err_query_response.set_error(format!("Error parsing JSON request: {}", err));
            return Ok(err_query_response);
        }
    };

    // Get the method from the JSON request
    let method = match json_request.get("method") {
        None => {        
            let mut err_query_response = QueryResponse::new();
            err_query_response.set_error("Method field not provided".to_owned());
            return Ok(err_query_response);    
        },
        Some(v) => {
            match v.as_str() {
                None => {
                    let mut err_query_response = QueryResponse::new();
                    err_query_response.set_error("Method field is not a string".to_owned());
                    return Ok(err_query_response);            
                }
                Some(s) => {
                    s
                }
            }
        }
    };

    // Either block or modify the request based on the method
    match method {
        "get_block_object" | 
        "get_txo" |
        "get_txo_object" | 
        "get_transaction" | 
        "get_transaction_object" => {
            // Whitelisted methods that pass through as-is
        },
        "get_balance" => {
            // Add / override account id

        }
        _ => {
            // Method is not on the whitelist
            let mut err_query_response = QueryResponse::new();
            err_query_response.set_error("Method is not valid".to_owned());
            return Ok(err_query_response);            
        }
    }

    let client = reqwest::blocking::Client::new();
    let json_response_result = client.post(wallet_service_uri)
        .json(&json_request)
        .send();

    let res = json_response_result.map_err(|err| {
        grpcio::Error::RpcFailure(RpcStatus::new(
            RpcStatusCode::INTERNAL,
            Some(format!("json serialization error: {}", err)),
        ))
    })?;

    let response_bytes = res.bytes().unwrap();

    let encrypted_payload = crypto::encrypt(mirror_key, &response_bytes).map_err(|_err| {
        grpcio::Error::RpcFailure(RpcStatus::new(
            RpcStatusCode::INTERNAL,
            Some("Encryption failed".into()),
        ))
    })?;

    let mut encrypted_response = EncryptedResponse::new();
    encrypted_response.set_payload(encrypted_payload);

    let mut mirror_response = QueryResponse::new();
    mirror_response.set_encrypted_response(encrypted_response);
    Ok(mirror_response)
}

fn parse_duration_in_milliseconds(src: &str) -> Result<Duration, std::num::ParseIntError> {
    Ok(Duration::from_millis(u64::from_str(src)?))
}

fn load_public_key(src: &str) -> Result<RSAPublicKey, String> {
    let key_str = std::fs::read_to_string(src)
        .map_err(|err| format!("failed reading key file {}: {:?}", src, err))?;
    let pem = pem::parse(&key_str)
        .map_err(|err| format!("failed parsing key file {}: {:?}", src, err))?;
    Ok(RSAPublicKey::try_from(pem)
        .map_err(|err| format!("failed loading key file {}: {:?}", src, err))?)
}
