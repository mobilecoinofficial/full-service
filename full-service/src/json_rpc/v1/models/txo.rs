// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the Txo object.

use crate::db::{self, txo::TxoInfo};
use serde_derive::{Deserialize, Serialize};
use serde_json::{json, Map};
use std::{convert::TryFrom, fmt, str::FromStr};

pub enum TxoStatus {
    Orphaned,
    Pending,
    Secreted,
    Spent,
    Unspent,
}

impl fmt::Display for TxoStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TxoStatus::Orphaned => write!(f, "txo_status_orphaned"),
            TxoStatus::Pending => write!(f, "txo_status_pending"),
            TxoStatus::Secreted => write!(f, "txo_status_secreted"),
            TxoStatus::Spent => write!(f, "txo_status_spent"),
            TxoStatus::Unspent => write!(f, "txo_status_unspent"),
        }
    }
}

impl FromStr for TxoStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "txo_status_secreted" => Ok(TxoStatus::Secreted),
            "txo_status_unspent" => Ok(TxoStatus::Unspent),
            "txo_status_pending" => Ok(TxoStatus::Pending),
            "txo_status_spent" => Ok(TxoStatus::Spent),
            "txo_status_orphaned" => Ok(TxoStatus::Orphaned),
            _ => Err(format!("Unknown TxoStatus: {s}")),
        }
    }
}

impl TryFrom<TxoStatus> for db::txo::TxoStatus {
    type Error = String;

    fn try_from(status: TxoStatus) -> Result<Self, Self::Error> {
        match status {
            TxoStatus::Orphaned => Ok(db::txo::TxoStatus::Orphaned),
            TxoStatus::Pending => Ok(db::txo::TxoStatus::Pending),
            TxoStatus::Secreted => Ok(db::txo::TxoStatus::Secreted),
            TxoStatus::Spent => Ok(db::txo::TxoStatus::Spent),
            TxoStatus::Unspent => Ok(db::txo::TxoStatus::Unspent),
        }
    }
}

impl From<&db::txo::TxoStatus> for TxoStatus {
    fn from(src: &db::txo::TxoStatus) -> Self {
        match src {
            db::txo::TxoStatus::Orphaned => TxoStatus::Orphaned,
            db::txo::TxoStatus::Pending => TxoStatus::Pending,
            db::txo::TxoStatus::Spent => TxoStatus::Spent,
            db::txo::TxoStatus::Unspent => TxoStatus::Unspent,
            db::txo::TxoStatus::Unverified => TxoStatus::Unspent,
            db::txo::TxoStatus::Secreted => TxoStatus::Secreted,
            db::txo::TxoStatus::Created => TxoStatus::Unspent,
        }
    }
}

pub enum TxoType {
    Minted,
    Received,
}

impl fmt::Display for TxoType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TxoType::Minted => write!(f, "txo_type_minted"),
            TxoType::Received => write!(f, "txo_type_received"),
        }
    }
}

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
    pub txo_id_hex: String,

    /// Available pico MOB for this account at the current account_block_height.
    /// If the account is syncing, this value may change.
    pub value_pmob: String,

    /// Unique identifier for the recipient associated account. Only available
    /// if direction is "sent".
    pub recipient_address_id: Option<String>,

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
}

impl From<&TxoInfo> for Txo {
    fn from(txo_info: &TxoInfo) -> Self {
        let mut account_status_map: Map<String, serde_json::Value> = Map::new();

        let status = TxoStatus::from(&txo_info.status);

        if let Some(account_id) = &txo_info.txo.account_id {
            account_status_map.insert(
                account_id.clone(),
                json!({"txo_type": TxoType::Received.to_string(), "txo_status": status.to_string()}),
            );
        }

        Txo {
            object: "txo".to_string(),
            txo_id_hex: txo_info.txo.id.clone(),
            value_pmob: (txo_info.txo.value as u64).to_string(),
            recipient_address_id: None,
            received_block_index: txo_info.txo.received_block_index.map(|i| i.to_string()),
            spent_block_index: txo_info.txo.spent_block_index.map(|i| i.to_string()),
            is_spent_recovered: false,
            received_account_id: txo_info.txo.clone().account_id,
            minted_account_id: None,
            account_status_map,
            target_key: hex::encode(&txo_info.txo.target_key),
            public_key: hex::encode(&txo_info.txo.public_key),
            e_fog_hint: hex::encode(&txo_info.txo.e_fog_hint),
            subaddress_index: txo_info.txo.subaddress_index.map(|i| i.to_string()),
            assigned_address: None,
            key_image: txo_info.txo.key_image.as_ref().map(hex::encode),
            confirmation: txo_info.txo.confirmation.as_ref().map(hex::encode),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db,
        db::{account::AccountModel, models::Account, txo::TxoModel},
        test_utils::{create_test_received_txo, WalletDbTestContext, MOB},
    };
    use mc_account_keys::{AccountKey, RootIdentity};
    use mc_common::logger::{test_with_logger, Logger};
    use mc_transaction_core::{tokens::Mob, Amount, Token};
    use mc_util_from_random::FromRandom;
    use rand::{rngs::StdRng, SeedableRng};

    #[test_with_logger]
    fn test_display_txo_in_origin(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let db_test_context = WalletDbTestContext::default();
        let wallet_db = db_test_context.get_db_instance(logger);

        let root_id = RootIdentity::from_random(&mut rng);
        let account_key = AccountKey::from(&root_id);
        let (_account_id_hex, _public_address_b58) = Account::create_from_root_entropy(
            &root_id.root_entropy,
            Some(1),
            None,
            None,
            "Alice's Main Account",
            "".to_string(),
            "".to_string(),
            None,
            &mut wallet_db.get_pooled_conn().unwrap(),
        )
        .unwrap();

        // Amount in origin block TXO is 250_000_000 MOB / 16
        let (txo_hex, _txo, _key_image) = create_test_received_txo(
            &account_key,
            0,
            Amount::new(15_625_000 * MOB, Mob::ID),
            0,
            &mut rng,
            &wallet_db,
        );

        let txo_details = db::models::Txo::get(&txo_hex, &mut wallet_db.get_pooled_conn().unwrap())
            .expect("Could not get Txo");
        let txo_status = txo_details
            .status(&mut wallet_db.get_pooled_conn().unwrap())
            .unwrap();
        let txo_memo = txo_details
            .memo(&mut wallet_db.get_pooled_conn().unwrap())
            .unwrap();
        assert_eq!(txo_details.value as u64, 15_625_000 * MOB);
        let json_txo = Txo::from(&TxoInfo {
            txo: txo_details,
            status: txo_status,
            memo: txo_memo,
        });
        assert_eq!(json_txo.value_pmob, "15625000000000000000");
    }
}
