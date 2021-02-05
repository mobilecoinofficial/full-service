// Copyright (c) 2020-2021 MobileCoin Inc.

use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct Account {
    pub object: String,
    pub account_id: String,
    pub name: String,
    pub network_height: String,
    pub local_height: String,
    pub account_height: String,
    pub is_synced: bool,
    pub available_pmob: String,
    pub pending_pmob: String,
    pub main_address: String,
    pub next_subaddress_index: String,
    pub recovery_mode: bool,
}
