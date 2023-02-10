// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the Public Address object.

use serde_derive::{Deserialize, Serialize};

/// A public mobilecoin address
#[derive(Deserialize, PartialEq, Eq, Serialize, Default, Debug, Clone)]
pub struct PublicAddress {
    /// The view public key for this address.
    pub view_public_key: String,

    /// The spend public key for the address.
    pub spend_public_key: String,

    /// The fog report url for this address.
    pub fog_report_url: Option<String>,

    /// The fog report id for this address.
    pub fog_report_id: Option<String>,

    /// The fog authority signature over the report id.
    pub fog_authority_sig: Option<String>,
}

impl From<&mc_account_keys::PublicAddress> for PublicAddress {
    fn from(src: &mc_account_keys::PublicAddress) -> PublicAddress {
        PublicAddress {
            view_public_key: hex::encode(src.view_public_key().to_bytes()),
            spend_public_key: hex::encode(src.spend_public_key().to_bytes()),
            fog_report_url: src.fog_report_url().map(|url| url.to_string()),
            fog_report_id: src.fog_report_id().map(|id| id.to_string()),
            fog_authority_sig: src.fog_authority_sig().map(hex::encode),
        }
    }
}
