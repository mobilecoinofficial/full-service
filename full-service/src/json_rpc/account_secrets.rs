// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the Account Secrets object.

use crate::{db::models::Account, json_rpc::account_key::AccountKey};

use serde_derive::{Deserialize, Serialize};
use std::convert::TryFrom;

/// The AccountSecrets contains the entropy and the account key derived from
/// that entropy.
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct AccountSecrets {
    /// String representing the object's type. Objects of the same type share
    /// the same value.
    pub object: String,

    /// The account ID for this account key in the wallet database.
    pub account_id: String,

    /// The entropy from which this account key was derived, hex-encoded.
    ///
    /// Optional because an account can be created from only the AccountKey.
    pub entropy: String,

    ///  Private key for receiving and spending MobileCoin.
    pub account_key: AccountKey,
}

impl TryFrom<&Account> for AccountSecrets {
    type Error = String;

    fn try_from(src: &Account) -> Result<AccountSecrets, String> {
        let account_key: mc_account_keys::AccountKey = mc_util_serial::decode(&src.account_key)
            .map_err(|err| format!("Could not decode account key from database: {:?}", err))?;
        Ok(AccountSecrets {
            object: "account_secrets".to_string(),
            account_id: src.account_id_hex.clone(),
            entropy: hex::encode(&src.entropy),
            account_key: AccountKey::try_from(&account_key).map_err(|err| {
                format!(
                    "Could not convert account_key to json_rpc representation: {:?}",
                    err
                )
            })?,
        })
    }
}
