// Copyright (c) 2020-2025 MobileCoin Inc.

use crate::{
    db::{account::AccountModel, models::Account},
    error::WalletTransactionBuilderError,
    service::hardware_wallet::HardwareWalletAuthenticatedMemoHmacSigner,
};
use mc_account_keys::{AccountKey, PublicAddress, ViewAccountKey};
use mc_transaction_builder::{
    BurnRedemptionMemoBuilder, EmptyMemoBuilder, MemoBuilder, RTHMemoBuilder,
};
use mc_transaction_extra::{AuthenticatedMemoHmacSigner, BurnRedemptionMemo, SenderMemoCredential};
use mc_util_serial::BigArray;
use serde::{Deserialize, Serialize};
use std::{boxed::Box, convert::TryFrom, sync::Arc};

/// This represents the different types of Transaction Memos that can be used in
/// a given transaction
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum TransactionMemo {
    /// Empty Transaction Memo.
    #[default]
    Empty,

    /// Recoverable Transaction History memo
    RTH {
        /// Subaddress index to generate the sender memo credential
        /// from.
        subaddress_index: u64,

        /// The public address that matches the subaddress index.
        #[serde(with = "crate::util::b58::public_address_b58")]
        sender_credentials_identify_as: PublicAddress,
    },

    /// Recoverable Transaction History memo with a payment intent id
    RTHWithPaymentIntentId {
        /// Subaddress index to generate the sender memo credential
        /// from.
        subaddress_index: u64,

        /// The payment intent id to include in the memo.
        payment_intent_id: u64,
    },

    /// Recoverable Transaction History memo with a payment request id
    RTHWithPaymentRequestId {
        /// Subaddress index to generate the sender memo credential
        /// from.
        subaddress_index: u64,

        /// The payment request id to include in the memo.
        payment_request_id: u64,
    },

    /// Burn Redemption memo, with an optional 64 byte redemption memo hex
    /// string.
    #[serde(with = "BigArray")]
    BurnRedemption([u8; BurnRedemptionMemo::MEMO_DATA_LEN]),
}

impl TransactionMemo {
    pub fn memo_builder(
        &self,
        signer_credentials: &TransactionMemoSignerCredentials,
    ) -> Result<Box<dyn MemoBuilder + Send + Sync>, WalletTransactionBuilderError> {
        match self {
            Self::Empty => Ok(Box::<EmptyMemoBuilder>::default()),
            Self::BurnRedemption(memo_data) => {
                let mut memo_builder = BurnRedemptionMemoBuilder::new(*memo_data);
                memo_builder.enable_destination_memo();
                Ok(Box::new(memo_builder))
            }
            Self::RTH {
                subaddress_index,
                sender_credentials_identify_as,
            } => {
                let memo_builder = generate_rth_memo_builder(
                    *subaddress_index,
                    sender_credentials_identify_as,
                    signer_credentials,
                )?;
                Ok(Box::new(memo_builder))
            }
            Self::RTHWithPaymentIntentId {
                subaddress_index,
                payment_intent_id,
            } => {
                todo!()

                // let mut memo_builder =
                //     generate_rth_memo_builder(*subaddress_index,
                // signer_credentials)?; memo_builder.
                // set_payment_intent_id(*payment_intent_id);
                // Ok(Box::new(memo_builder))
            }
            Self::RTHWithPaymentRequestId {
                subaddress_index,
                payment_request_id,
            } => {
                todo!()

                // let mut memo_builder =
                //     generate_rth_memo_builder(*subaddress_index,
                // signer_credentials)?; memo_builder.
                // set_payment_request_id(*payment_request_id);
                // Ok(Box::new(memo_builder))
            }
        }
    }
}

fn generate_rth_memo_builder(
    subaddress_index: u64,
    sender_credentials_identify_as: &PublicAddress,
    signer_credentials: &TransactionMemoSignerCredentials,
) -> Result<RTHMemoBuilder, WalletTransactionBuilderError> {
    let mut memo_builder = RTHMemoBuilder::default();

    match signer_credentials {
        TransactionMemoSignerCredentials::None => {
            // No authenticated sender
        }

        TransactionMemoSignerCredentials::ViewOnly(_) => {
            return Err(WalletTransactionBuilderError::RTHUnavailableForViewOnlyAccounts);
        }

        TransactionMemoSignerCredentials::Local(account_key) => {
            memo_builder.set_sender_credential(
                SenderMemoCredential::new_from_address_and_spend_private_key(
                    sender_credentials_identify_as,
                    account_key.subaddress_spend_private(subaddress_index),
                ),
            );
        }

        TransactionMemoSignerCredentials::HardwareWallet(view_account_key) => {
            let signer: Arc<Box<dyn AuthenticatedMemoHmacSigner + 'static + Send + Sync>> =
                Arc::new(Box::new(HardwareWalletAuthenticatedMemoHmacSigner::new(
                    sender_credentials_identify_as,
                    subaddress_index,
                )?));
            memo_builder.set_authenticated_sender_hmac_signer(signer);
        }
    };

    memo_builder.enable_destination_memo();

    Ok(memo_builder)
}

/// Credentials used to sign a transaction memo.
pub enum TransactionMemoSignerCredentials {
    /// No credentials available.
    None,

    /// Local account credentials.
    Local(AccountKey),

    /// Hardware wallet credentials.
    HardwareWallet(ViewAccountKey),

    /// View only account (not managed by hardware wallet)
    ViewOnly(ViewAccountKey),
}

impl TryFrom<&Account> for TransactionMemoSignerCredentials {
    type Error = WalletTransactionBuilderError;

    fn try_from(account: &Account) -> Result<Self, Self::Error> {
        if account.view_only {
            if account.managed_by_hardware_wallet {
                Ok(Self::HardwareWallet(account.view_account_key()?))
            } else {
                Ok(Self::ViewOnly(account.view_account_key()?))
            }
        } else {
            Ok(Self::Local(account.account_key()?))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mc_account_keys::AccountKey;
    use mc_util_from_random::FromRandom;
    use rand::{rngs::StdRng, SeedableRng};

    #[test]
    fn test_rth_memo_b58_roundtrip() {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let account_key = AccountKey::from_random(&mut rng);
        let public_address = account_key.subaddress(5);
        let b58_address =
            crate::util::b58::b58_encode_public_address(&public_address).expect("Failed to encode");

        let memo = TransactionMemo::RTH {
            subaddress_index: 5,
            sender_credentials_identify_as: public_address.clone(),
        };

        let serialized = serde_json::to_string(&memo).expect("Failed to serialize");

        let expected_json = format!(
            r#"{{"RTH":{{"subaddress_index":5,"sender_credentials_identify_as":"{}"}}}}"#,
            b58_address
        );
        assert_eq!(
            serialized, expected_json,
            "JSON serialization did not match expected format"
        );

        let deserialized: TransactionMemo =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(memo, deserialized, "Round-trip serialization failed");
    }
}
