// Copyright (c) 2018-2020 MobileCoin Inc.

//! Serializeable data types for the wallet service API.
use serde_derive::{Deserialize, Serialize};

// CreateAccount
//

#[derive(Deserialize, Default)]
pub struct WalletCreateAccountRequest {
    pub method: String,
    pub params: WalletCreateAccountParams,
}

#[derive(Deserialize, Serialize, Default)]
pub struct WalletCreateAccountParams {
    pub title: String,
    pub body: String,
}

#[derive(Serialize, Default)]
pub struct WalletCreateAccountResponse {
    pub public_address: String,
    pub entropy: String,
    pub account_id: String,
}

/*
impl From<&mc_mobilecoind_api::CreateAccountResponse> for WalletCreateAccountResponse {
    fn from(src: &mc_mobilecoind_api::CreateAccountResponse) -> Self {
        Self {
            public_address: src.public_address.to_string(),
            entropy: hex::encode(src.entropy.clone()),
            account_id: hex::encode(src.account_id.clone()),
        }
    }
}
*/
