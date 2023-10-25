// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the Txo object.

use crate::{db::txo::TxoInfo, json_rpc::v2::models::memo::Memo};

use serde_derive::{Deserialize, Serialize};

/// An Txo in the wallet.
///
/// An Txo is associated with one or two accounts, and can be categorized with
/// different statuses and types in relation to those accounts.
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct Txo {
    /// Unique identifier for the Txo. Constructed from the contents of the
    /// TxOut in the ledger representation.
    pub id: String,

    /// the txo's value
    pub value: String,

    /// the txo's token id
    pub token_id: String,

    /// Block index in which the txo was received by an account.
    pub received_block_index: Option<String>,

    /// Block index in which the txo was spent by an account.
    pub spent_block_index: Option<String>,

    /// The account_id for the account which has received this TXO. This account
    /// has spend authority.
    pub account_id: Option<String>,

    /// The status of this txo
    pub status: String,

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

    /// A fingerprint of the txo derived from your private spend key materials,
    /// required to spend a Txo.
    pub key_image: Option<String>,

    /// A confirmation number that the sender of the Txo can provide to verify
    /// that they participated in the construction of this Txo.
    pub confirmation: Option<String>,

    /// Shared secret that's used to mask the private keys associated with the
    /// amounts in a transaction
    pub shared_secret: Option<String>,

    /// The memo associated with this Txo.
    pub memo: Memo,
}

impl From<&TxoInfo> for Txo {
    fn from(txo_info: &TxoInfo) -> Self {
        Txo {
            id: txo_info.txo.id.clone(),
            value: (txo_info.txo.value as u64).to_string(),
            token_id: (txo_info.txo.token_id as u64).to_string(),
            received_block_index: txo_info
                .txo
                .received_block_index
                .map(|x| (x as u64).to_string()),
            spent_block_index: txo_info
                .txo
                .spent_block_index
                .map(|x| (x as u64).to_string()),
            account_id: txo_info.txo.account_id.clone(),
            status: txo_info.status.to_string(),
            target_key: hex::encode(&txo_info.txo.target_key),
            public_key: hex::encode(&txo_info.txo.public_key),
            e_fog_hint: hex::encode(&txo_info.txo.e_fog_hint),
            subaddress_index: txo_info
                .txo
                .subaddress_index
                .map(|s| (s as u64).to_string()),
            key_image: txo_info.txo.key_image.as_ref().map(hex::encode),
            confirmation: txo_info.txo.confirmation.as_ref().map(hex::encode),
            shared_secret: txo_info.txo.shared_secret.as_ref().map(hex::encode),
            memo: (&txo_info.memo).into(),
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

        let txo = db::models::Txo::get(&txo_hex, &mut wallet_db.get_pooled_conn().unwrap())
            .expect("Could not get Txo");
        let status = txo
            .status(&mut wallet_db.get_pooled_conn().unwrap())
            .unwrap();
        let memo = txo.memo(&mut wallet_db.get_pooled_conn().unwrap()).unwrap();

        let txo_info = TxoInfo { txo, status, memo };

        assert_eq!(txo_info.txo.value as u64, 15_625_000 * MOB);
        let json_txo = Txo::from(&txo_info);
        assert_eq!(json_txo.value, "15625000000000000000");
        assert_eq!(json_txo.token_id, "0");
    }
}
