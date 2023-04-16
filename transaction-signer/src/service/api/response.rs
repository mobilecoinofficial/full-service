use mc_full_service::json_rpc::json_rpc_response::JsonCommandResponse as JsonCommandResponseTrait;

use serde::{Deserialize, Serialize};

/// Responses from the Full Service Wallet.
#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
#[allow(non_camel_case_types)]
#[allow(clippy::large_enum_variant)]
pub enum JsonCommandResponse {}

impl JsonCommandResponseTrait for JsonCommandResponse {}
