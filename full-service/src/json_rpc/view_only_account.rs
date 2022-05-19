// Copyright (c) 2020-2022 MobileCoin Inc.

//! API definition for the View Only Account object.

use crate::{
    db,
    json_rpc::{
        json_rpc_request::JsonCommandRequest,
        view_only_subaddress::{ViewOnlySubaddressJSON, ViewOnlySubaddressesJSON},
    },
    util::encoding_helpers::ristretto_to_hex,
};
use serde_derive::{Deserialize, Serialize};
use std::convert::TryFrom;

/// An view-only-account in the wallet.
///
/// A view only account is associated with one private view key
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct ViewOnlyAccountJSON {
    /// String representing the object's type. Objects of the same type share
    /// the same value.
    pub object: String,

    /// Display name for the account.
    pub account_id: String,

    /// Display name for the account.
    pub name: String,

    /// Index of the first block when this account may have received funds.
    /// No transactions before this point will be synchronized.
    pub first_block_index: String,

    /// Index of the next block this account needs to sync.
    pub next_block_index: String,

    pub main_subaddress_index: String,

    pub change_subaddress_index: String,

    pub next_subaddress_index: String,
}

impl From<&db::models::ViewOnlyAccount> for ViewOnlyAccountJSON {
    fn from(src: &db::models::ViewOnlyAccount) -> ViewOnlyAccountJSON {
        ViewOnlyAccountJSON {
            object: "view_only_account".to_string(),
            name: src.name.clone(),
            account_id: src.account_id_hex.clone(),
            first_block_index: (src.first_block_index as u64).to_string(),
            next_block_index: (src.next_block_index as u64).to_string(),
            main_subaddress_index: (src.main_subaddress_index as u64).to_string(),
            change_subaddress_index: (src.change_subaddress_index as u64).to_string(),
            next_subaddress_index: (src.next_subaddress_index as u64).to_string(),
        }
    }
}

impl From<&db::models::Account> for ViewOnlyAccountJSON {
    fn from(src: &db::models::Account) -> ViewOnlyAccountJSON {
        ViewOnlyAccountJSON {
            object: "view_only_account".to_string(),
            name: src.name.clone(),
            account_id: src.account_id_hex.clone(),
            first_block_index: (src.first_block_index as u64).to_string(),
            next_block_index: (src.next_block_index as u64).to_string(),
            main_subaddress_index: (src.main_subaddress_index as u64).to_string(),
            change_subaddress_index: (src.change_subaddress_index as u64).to_string(),
            next_subaddress_index: (src.next_subaddress_index as u64).to_string(),
        }
    }
}

/// private view key for the account
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct ViewOnlyAccountSecretsJSON {
    /// The private key used for viewing transactions for this account
    pub object: String,
    pub view_private_key: String,
    pub account_id: String,
}

impl TryFrom<&db::models::ViewOnlyAccount> for ViewOnlyAccountSecretsJSON {
    type Error = String;

    fn try_from(src: &db::models::ViewOnlyAccount) -> Result<ViewOnlyAccountSecretsJSON, String> {
        Ok(ViewOnlyAccountSecretsJSON {
            object: "view_only_account_secrets".to_string(),
            account_id: src.account_id_hex.clone(),
            view_private_key: hex::encode(src.view_private_key.as_slice()),
        })
    }
}

impl TryFrom<&db::models::Account> for ViewOnlyAccountSecretsJSON {
    type Error = String;

    fn try_from(src: &db::models::Account) -> Result<ViewOnlyAccountSecretsJSON, String> {
        let account_key: mc_account_keys::AccountKey = mc_util_serial::decode(&src.account_key)
            .map_err(|err| format!("Could not decode account key from database: {:?}", err))?;

        Ok(ViewOnlyAccountSecretsJSON {
            object: "view_only_account_secrets".to_string(),
            account_id: src.account_id_hex.clone(),
            view_private_key: ristretto_to_hex(account_key.view_private_key()),
        })
    }
}

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct ViewOnlyAccountImportPackageJSON {
    pub object: String,
    pub account: ViewOnlyAccountJSON,
    pub secrets: ViewOnlyAccountSecretsJSON,
    pub subaddresses: ViewOnlySubaddressesJSON,
}

impl TryFrom<&db::account::ViewOnlyAccountImportPackage> for JsonCommandRequest {
    type Error = String;

    fn try_from(
        src: &db::account::ViewOnlyAccountImportPackage,
    ) -> Result<JsonCommandRequest, String> {
        let account = ViewOnlyAccountJSON::from(&src.account);
        let secrets = ViewOnlyAccountSecretsJSON::try_from(&src.account)?;
        let subaddresses = src
            .subaddresses
            .iter()
            .map(ViewOnlySubaddressJSON::from)
            .collect();

        Ok(JsonCommandRequest::import_view_only_account {
            account,
            secrets,
            subaddresses,
        })
    }
}
