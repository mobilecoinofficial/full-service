// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the Txo object.

use crate::db::txo::TxoDetails;
use serde_derive::{Deserialize, Serialize};
use serde_json::Map;

/// An Txo in the wallet.
///
/// An Txo is associated with one or two accounts, and can be categorized with
/// different statuses and types in relation to those accounts.
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct Txo {
    /// String representing the object's type. Objects of the same type share
    /// the same value.
    pub object: String,

    /// Unique identifier for the Txo. Constructed from the contents of the
    /// TxOut in the ledger representation.
    pub txo_id: String,

    /// Available pico MOB for this account at the current account_block_index.
    /// If the account is syncing, this value may change.
    pub value_pmob: String,

    /// Block index in which the txo was received by an account.
    pub received_block_index: Option<String>,

    /// Block index in which the txo was spent by an account.
    pub spent_block_index: Option<String>,

    /// Flag that indicates if the spent_block_index was recovered from the
    /// ledger. This value is null if the txo is unspent. If true, some
    /// information may not be available on the txo without user input. If true,
    /// the confirmation number will be null without user input.
    pub is_spent_recovered: bool, // FIXME: WS-16 is_spent_recovered

    /// The account_id for the account which has received this TXO. This account
    /// has spend authority.
    pub received_account_id: Option<String>,

    /// The account_id for the account which minted this Txo.
    pub minted_account_id: Option<String>,

    /// A normalized hash mapping account_id to account objects. Keys include
    /// "type" and "status".
    ///
    /// * `txo_type`: With respect to this account, the Txo may be
    /// "minted" or "received".
    ///
    /// * `txo_status`: With respect to this account, the Txo may be "unspent",
    ///   "pending", "spent", "secreted" or "orphaned". For received Txos
    ///   received as an assigned address, the lifecycle is "unspent" ->
    ///   "pending" -> "spent". For outbound, minted Txos, we cannot monitor its
    ///   received lifecycle status with respect to the minting account, we note
    ///   its status as "secreted". If a Txo is received at an address
    ///   unassigned (likely due to a recovered account or using the account on
    ///   another client), the Txo is considered "orphaned" until its address is
    ///   calculated -- in this case, there are manual ways to discover the
    ///   missing assigned address for orphaned Txos or to recover an entire
    ///   account.
    pub account_status_map: Map<String, serde_json::Value>,

    /// A cryptographic key for this Txo.
    pub target_key: String,

    /// The public key for this txo, can be used as an identifier to find the
    /// txo in the ledger.
    pub public_key: String,

    /// The encrypted fog hint for this Txo.
    pub e_fog_hint: String,

    /// The assigned subaddress index for this Txo with respect to its received
    /// account.
    pub subaddress_index: Option<String>,

    /// The address corresponding to the subaddress index which was assigned as
    /// an intended sender for this Txo.
    pub assigned_address: Option<String>,

    /// A fingerprint of the txo derived from your private spend key materials,
    /// required to spend a Txo.
    pub key_image: Option<String>,

    /// A confirmation number that the sender of the Txo can provide to verify
    /// that they participated in the construction of this Txo.
    pub confirmation: Option<String>,

    /// The value to offset pagination requests. Requests will exclude all list
    /// items up to and including this object.
    pub offset_count: i32,
}

impl From<&TxoDetails> for Txo {
    fn from(txo_details: &TxoDetails) -> Txo {
        let mut account_status_map: Map<String, serde_json::Value> = Map::new();

        if let Some(received) = txo_details.received_to_account.clone() {
            account_status_map.insert(
                received.account_id_hex,
                json!({"txo_type": received.txo_type, "txo_status": received.txo_status}).into(),
            );
        }

        if let Some(spent) = txo_details.minted_from_account.clone() {
            account_status_map.insert(
                spent.account_id_hex,
                json!({"txo_type": spent.txo_type, "txo_status": spent.txo_status}).into(),
            );
        }

        Txo {
            object: "txo".to_string(),
            txo_id: txo_details.txo.txo_id_hex.clone(),
            value_pmob: txo_details.txo.value.to_string(),
            received_block_index: txo_details.txo.received_block_index.map(|x| x.to_string()),
            spent_block_index: txo_details.txo.spent_block_index.map(|x| x.to_string()),
            is_spent_recovered: false,
            received_account_id: txo_details
                .received_to_account
                .as_ref()
                .map(|a| a.account_id_hex.clone()),
            minted_account_id: txo_details
                .clone()
                .minted_from_account
                .as_ref()
                .map(|a| a.account_id_hex.clone()),
            account_status_map,
            target_key: hex::encode(&txo_details.txo.target_key),
            public_key: hex::encode(&txo_details.txo.public_key),
            e_fog_hint: hex::encode(&txo_details.txo.e_fog_hint),
            subaddress_index: txo_details.txo.subaddress_index.map(|s| s.to_string()),
            assigned_address: txo_details
                .received_to_assigned_subaddress
                .clone()
                .map(|a| a.assigned_subaddress_b58),
            key_image: txo_details.txo.key_image.as_ref().map(|k| hex::encode(&k)),
            confirmation: txo_details.txo.confirmation.as_ref().map(hex::encode),
            offset_count: txo_details.txo.id,
        }
    }
}
