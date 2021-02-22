// Copyright (c) 2020-2021 MobileCoin Inc.

//! The JSON RPC 2.0 Requests to the Wallet API for Full Service.
//!
//! API v2

use crate::json_rpc::api_v1::wallet_api::JsonCommandRequestV1;

use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

// FIXME: Update
/// Help string when invoking GET on the wallet endpoint.
pub fn help_str_v2() -> String {
    let mut help_str = "Please use json data to choose wallet commands. For example, \n\ncurl -s localhost:9090/wallet -d '{\"method\": \"create_account\", \"params\": {\"name\": \"Alice\"}}' -X POST -H 'Content-type: application/json'\n\nAvailable commands are:\n\n".to_owned();
    for e in JsonCommandRequestV2::iter() {
        help_str.push_str(&format!("{:?}\n\n", e));
    }
    help_str
}

/// JSON RPC 2.0 Request.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[allow(non_camel_case_types)]
pub struct JsonCommandRequest {
    /// The method to be invoked on the server.
    pub method: String,

    /// The parameters to be provided to the method.
    ///
    /// Optional, as some methods do not take parameters.
    pub params: Option<serde_json::Value>,

    /// The JSON RPC Version (Should always be 2.0)
    ///
    /// Optional for backwards compatibility because the previous version of
    /// this API (v1) did not require the jsonrpc parameter.
    pub jsonrpc: Option<String>,

    /// The ID to be associated with this request.
    ///
    /// Optional because a "notify" method does not need to correlate an ID on
    /// the response.
    pub id: Option<u32>,

    /// The Full Service Wallet API version.
    ///
    /// Optional: If omitted, assumes V1.
    pub api_version: Option<String>,
}

impl TryFrom<&JsonCommandRequest> for JsonCommandRequestV1 {
    type Error = String;

    fn try_from(src: &JsonCommandRequest) -> Result<JsonCommandRequestV1, String> {
        let src_json: serde_json::Value = serde_json::json!(src);
        Ok(serde_json::from_value(src_json).map_err(|e| format!("Could not get value {:?}", e))?)
    }
}

impl TryFrom<&JsonCommandRequest> for JsonCommandRequestV2 {
    type Error = String;

    fn try_from(src: &JsonCommandRequest) -> Result<JsonCommandRequestV2, String> {
        let src_json: serde_json::Value = serde_json::json!(src);
        Ok(serde_json::from_value(src_json).map_err(|e| format!("Could not get value {:?}", e))?)
    }
}

/// Requests to the Full Service Wallet Service.
#[derive(Deserialize, Serialize, EnumIter, Debug)]
#[serde(tag = "method", content = "params")]
#[allow(non_camel_case_types)]
pub enum JsonCommandRequestV2 {
    create_account {
        name: Option<String>,
        first_block: Option<String>,
    },
}
