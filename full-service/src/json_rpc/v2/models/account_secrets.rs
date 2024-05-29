// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the Account Secrets object.

use crate::{
    db::models::Account,
    json_rpc::v2::models::account_key::{AccountKey, ViewAccountKey},
};

use bip39::{Language, Mnemonic};
use redact::{expose_secret, Secret};
use serde_derive::{Deserialize, Serialize};
use std::convert::TryFrom;

/// The AccountSecrets contains the entropy and the account key derived from
/// that entropy.
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct AccountSecrets {
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
    #[serde(serialize_with = "expose_secret")]
    pub account_key: Secret<Option<AccountKey>>,

    ///  Private keys for receiving and spending MobileCoin.
    #[serde(serialize_with = "expose_secret")]
    pub view_account_key: Secret<Option<ViewAccountKey>>,

    /// Indicates that the account requires a spend_subaddress be
    /// specified when building a transaction in order to keep subaddress
    /// balances correct.
    pub require_spend_subaddress: bool,
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
                account_id: src.id.clone(),
                name: src.name.clone(),
                entropy: Secret::new(None),
                mnemonic: Secret::new(None),
                key_derivation_version: src.key_derivation_version.to_string(),
                account_key: Secret::new(None),
                view_account_key: Secret::new(Some(ViewAccountKey::from(&view_account_key))),
                require_spend_subaddress: src.require_spend_subaddress,
            })
        } else {
            let account_key: mc_account_keys::AccountKey = mc_util_serial::decode(&src.account_key)
                .map_err(|err| format!("Could not decode account key from database: {err:?}"))?;

            let entropy = match src.key_derivation_version {
                1 => Secret::new(Some(hex::encode(
                    src.entropy.as_ref().ok_or("No entropy found")?,
                ))),
                _ => Secret::new(None),
            };

            let mnemonic = match src.key_derivation_version {
                2 => Secret::new(Some(
                    Mnemonic::from_entropy(
                        src.entropy.as_ref().ok_or("No entropy found")?,
                        Language::English,
                    )
                    .map_err(|err| format!("Could not create mnemonic: {err:?}"))?
                    .phrase()
                    .to_string(),
                )),
                _ => Secret::new(None),
            };

            Ok(AccountSecrets {
                name: src.name.clone(),
                account_id: src.id.clone(),
                entropy,
                mnemonic,
                key_derivation_version: src.key_derivation_version.to_string(),
                account_key: Secret::new(Some(AccountKey::try_from(&account_key).map_err(
                    |err| {
                        format!("Could not convert account_key to json_rpc representation: {err:?}")
                    },
                )?)),
                view_account_key: Secret::new(None),
                require_spend_subaddress: src.require_spend_subaddress,
            })
        }
    }
}
