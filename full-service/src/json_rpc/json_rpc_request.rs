// Copyright (c) 2020-2023 MobileCoin Inc.

//! The JSON RPC 2.0 Requests to the Wallet API for Full Service.

use serde::{Deserialize, Serialize};

/// JSON-RPC 2.0 Request.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[allow(non_camel_case_types)]
pub struct JsonRPCRequest {
    /// The method to be invoked on the server.
    pub method: String,

    /// The parameters to be provided to the method.
    ///
    /// Optional, as some methods do not take parameters.
    pub params: Option<serde_json::Value>,

    /// The JSON-RPC Version (Should always be 2.0)
    pub jsonrpc: String,

    /// The ID to be associated with this request.
    /// JSON-RPC Notification requests are not yet supported, so this field is
    /// not optional.
    pub id: serde_json::Value,
}
