// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the Account Secrets object.

use crate::{db::models::Account, json_rpc::account_key::AccountKey};

use bip39::{Language, Mnemonic};
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

    /// The mnemonic from which this account key was derived, as a list of BIP39
    /// words.
    pub mnemonic: String,

    ///  Private keys for receiving and spending MobileCoin.
    pub account_key: AccountKey,
}

impl TryFrom<&Account> for AccountSecrets {
    type Error = String;

    fn try_from(src: &Account) -> Result<AccountSecrets, String> {
        if src.key_derivation_version == 1 {
            Err("Not allowed to export secrets for legacy account".to_string())
        } else {
            let account_key: mc_account_keys::AccountKey = mc_util_serial::decode(&src.account_key)
                .map_err(|err| format!("Could not decode account key from database: {:?}", err))?;

            Ok(AccountSecrets {
                object: "account_secrets".to_string(),
                account_id: src.account_id_hex.clone(),
                mnemonic: Mnemonic::from_entropy(&src.entropy, Language::English)
                    .unwrap()
                    .phrase()
                    .to_string(),
                account_key: AccountKey::try_from(&account_key).map_err(|err| {
                    format!(
                        "Could not convert account_key to json_rpc representation: {:?}",
                        err
                    )
                })?,
            })
        }
    }
}
