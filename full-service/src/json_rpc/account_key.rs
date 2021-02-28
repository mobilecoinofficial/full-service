// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the Account Key object.

use serde_derive::{Deserialize, Serialize};
use std::convert::TryFrom;

/// The AccountKey contains a View keypair and a Spend keypair, used to
/// construct and receive transactions.
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct AccountKey {
    /// String representing the object's type. Objects of the same type share
    /// the same value.
    pub object: String,

    ///  Private key used for view-key matching, hex-encoded Ristretto bytes.
    pub view_private_key: String,

    /// Private key used for spending, hex-encoded Ristretto bytes.
    pub spend_private_key: String,

    /// Fog Report server url (if user has Fog service), empty string otherwise.
    pub fog_report_url: String,

    /// Fog Report Key (if user has Fog service), empty otherwise
    /// The key labelling the report to use, from among the several reports
    /// which might be served by the fog report server.
    pub fog_report_id: String,

    /// Fog Authority Subject Public Key Info (if user has Fog service),
    /// empty string otherwise.
    pub fog_authority_spki: String,
}

impl TryFrom<&mc_account_keys::AccountKey> for AccountKey {
    type Error = String;

    fn try_from(src: &mc_account_keys::AccountKey) -> Result<AccountKey, String> {
        Ok(AccountKey {
            object: "account_key".to_string(),
            view_private_key: hex::encode(mc_util_serial::encode(src.view_private_key())),
            spend_private_key: hex::encode(mc_util_serial::encode(src.spend_private_key())),
            fog_report_url: src.fog_report_url().unwrap_or("").to_string(),
            fog_report_id: src.fog_report_id().unwrap_or("").to_string(),
            fog_authority_spki: hex::encode(&src.fog_authority_spki().unwrap_or(&[])),
        })
    }
}
