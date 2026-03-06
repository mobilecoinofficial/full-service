// Copyright (c) 2018-2024 MobileCoin Inc.

//! Manages sending a webhook for synced accounts that have received deposits

use crate::db::account::AccountID;
use mc_common::logger::{log, Logger};

use crate::config::WebhookConfig;
use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderValue, CONTENT_TYPE},
};
use serde_json::json;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
};

pub struct WebhookThread {
    /// The main sync thread handle.
    join_handle: Option<thread::JoinHandle<()>>,

    /// Stop trigger, used to signal the thread to terminate.
    stop_requested: Arc<AtomicBool>,
}

impl WebhookThread {
    pub fn start(
        webhook_config: WebhookConfig,
        accounts_with_deposits: Arc<Mutex<HashMap<AccountID, bool>>>,
        logger: Logger,
    ) -> Self {
        // Start the webhook thread.

        let stop_requested = Arc::new(AtomicBool::new(false));
        let thread_stop_requested = stop_requested.clone();

        // Question: Should we consider only spawning a thread when there
        // have been received txos, and therefore something to send?
        // Answer: For now, we are ok with having this thread running all the time,
        // because it is lightweight enough. If we find that it is causing issues,
        // we will revisit. The solution would be to make this async, or to only
        // spawn the thread when there are txos to send. There may be an advantage to
        // leaving the connection open.

        let join_handle = Some(
            thread::Builder::new()
                .name("webhook".to_string())
                .spawn(move || {
                    log::debug!(logger, "Webhook thread started.");

                    let client = Client::builder()
                        .build()
                        .expect("Failed creating reqwest client");
                    let mut json_headers = HeaderMap::new();
                    json_headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

                    loop {
                        if thread_stop_requested.load(Ordering::SeqCst) {
                            log::debug!(logger, "WebhookThread stop requested.");
                            break;
                        }
                        // Gather the current accounts_to_send, then wipe the contents
                        let accounts_to_send: Vec<_> = accounts_with_deposits
                            .lock()
                            .unwrap()
                            .clone()
                            .iter()
                            .filter(|&(_k, &v)| v)
                            .map(|(k, _v)| k.clone())
                            .collect();

                        // Delete the keys that we're alerting on
                        for key in accounts_to_send.iter() {
                            log::debug!(logger, "Account to send: {:?}", key);
                            accounts_with_deposits.lock().unwrap().remove(key);
                        }

                        if !accounts_to_send.is_empty() {
                            // Question: will this keep the connection open? Or will it
                            // close the connection after this request?
                            match client
                                .post(webhook_config.url.clone())
                                .body(
                                    json!(
                                        {
                                            "accounts": accounts_to_send,
                                        }
                                    )
                                    .to_string(),
                                )
                                .send()
                            {
                                Ok(response) => match response.error_for_status() {
                                    Ok(_) => (),
                                    Err(e) => {
                                        log::error!(
                                            logger,
                                            "Failed getting webhook response: {:?}",
                                            e
                                        );
                                    }
                                },
                                Err(e) => {
                                    log::error!(logger, "Failed sending webhook request: {:?}", e);
                                }
                            }
                        }
                        // for new blocks from consensus
                        thread::sleep(webhook_config.poll_interval);
                    }
                })
                .expect("failed starting webhook thread"),
        );
        Self {
            join_handle,
            stop_requested,
        }
    }
    pub fn stop(&mut self) {
        self.stop_requested.store(true, Ordering::SeqCst);
        if let Some(join_handle) = self.join_handle.take() {
            join_handle.join().expect("WebhookThread join failed");
        }
    }
}

impl Drop for WebhookThread {
    fn drop(&mut self) {
        self.stop();
    }
}
