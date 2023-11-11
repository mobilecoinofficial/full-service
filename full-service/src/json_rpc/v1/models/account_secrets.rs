// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the Account Secrets object.

use crate::{
    db::models::Account,
    json_rpc::v1::models::account_key::{AccountKey, ViewAccountKey},
};

use bip39::{Language, Mnemonic};
use redact::{expose_secret, Secret};
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

    /// The name of this account
    pub name: String,

    /// The entropy from which this account key was derived, as a String
    /// (version 1)
    #[serde(serialize_with = "expose_secret")]
    pub entropy: Secret<Option<String>>,

    /// The mnemonic from which this account key was derived, as a String
    /// (version 2)
    #[serde(serialize_with = "expose_secret")]
    pub mnemonic: Secret<Option<String>>,

    /// The key derivation version that this mnemonic goes with
    pub key_derivation_version: String,

    ///  Private keys for receiving and spending MobileCoin.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_key: Option<AccountKey>,

    ///  Private keys for receiving and spending MobileCoin.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view_account_key: Option<ViewAccountKey>,
}

impl TryFrom<&Account> for AccountSecrets {
    type Error = String;

    fn try_from(src: &Account) -> Result<AccountSecrets, String> {
        if src.view_only {
            let view_account_key: mc_account_keys::ViewAccountKey =
                mc_util_serial::decode(&src.account_key).map_err(|err| {
                    format!("Could not decode account key from database: {err:?}")
                })?;

            Ok(AccountSecrets {
                object: "account_secrets".to_string(),
                account_id: src.id.clone(),
                name: src.name.clone(),
                entropy: Secret::new(None),
                mnemonic: Secret::new(None),
                key_derivation_version: src.key_derivation_version.to_string(),
                account_key: None,
                view_account_key: Some(ViewAccountKey::from(&view_account_key)),
            })
        } else {
            let account_key: mc_account_keys::AccountKey = mc_util_serial::decode(&src.account_key)
                .map_err(|err| format!("Could not decode account key from database: {err:?}"))?;

            let entropy = match src.key_derivation_version {
                1 => {
                    let entropy = src.entropy.as_ref().ok_or("No entropy found")?;
                    Some(hex::encode(entropy))
                }
                _ => None,
            };

            let mnemonic = match src.key_derivation_version {
                2 => {
                    let entropy = src.entropy.as_ref().ok_or("No entropy found")?;
                    Some(
                        Mnemonic::from_entropy(entropy, Language::English)
                            .map_err(|err| {
                                format!("Could not decode mnemonic from entropy: {err:?}")
                            })?
                            .phrase()
                            .to_string(),
                    )
                }
                _ => None,
            };

            Ok(AccountSecrets {
                object: "account_secrets".to_string(),
                name: src.name.clone(),
                account_id: src.id.clone(),
                entropy: entropy.into(),
                mnemonic: mnemonic.into(),
                key_derivation_version: src.key_derivation_version.to_string(),
                account_key: Some(AccountKey::try_from(&account_key).map_err(|err| {
                    format!(
                        "Could not convert account_key to json_rpc
                representation: {err:?}"
                    )
                })?),
                view_account_key: None,
            })
        }
    }
}
