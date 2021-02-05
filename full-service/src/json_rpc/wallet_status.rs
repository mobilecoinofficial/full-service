use alloc::string::String;
use serde_derive::{Deserialize, Serialize};
use serde_json::Map;

#[derive(Eq, PartialEq, Deserialize, Serialize, Default, Debug, Clone)]
pub struct WalletStatus {
    pub object: String,
    pub network_height: String,
    pub local_height: String,
    pub is_synced_all: bool,
    pub total_available_pmob: String,
    pub total_pending_pmob: String,
    pub account_ids: Vec<String>,
    pub account_map: Map<String, serde_json::Value>,
}
