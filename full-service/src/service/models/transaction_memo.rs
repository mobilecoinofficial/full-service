// Copyright (c) 2020-2025 MobileCoin Inc.

use crate::{
    db::{account::AccountModel, models::Account},
    error::WalletTransactionBuilderError,
    service::hardware_wallet::HardwareWalletAuthenticatedMemoHmacSigner,
};
use mc_account_keys::{AccountKey, ViewAccountKey, DEFAULT_SUBADDRESS_INDEX};
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
        /// Optional subaddress index to generate the sender memo credential
        /// from.
        subaddress_index: Option<u64>,
    },

    /// Recoverable Transaction History memo with a payment intent id
    RTHWithPaymentIntentId {
        /// Optional subaddress index to generate the sender memo credential
        /// from.
        subaddress_index: Option<u64>,

        /// The payment intent id to include in the memo.
        payment_intent_id: u64,
    },

    /// Recoverable Transaction History memo with a payment request id
    RTHWithPaymentRequestId {
        /// Optional subaddress index to generate the sender memo credential
        /// from.
        subaddress_index: Option<u64>,

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
            Self::RTH { subaddress_index } => {
                let memo_builder = generate_rth_memo_builder(subaddress_index, signer_credentials)?;
                Ok(Box::new(memo_builder))
            }
            Self::RTHWithPaymentIntentId {
                subaddress_index,
                payment_intent_id,
            } => {
                let mut memo_builder =
                    generate_rth_memo_builder(subaddress_index, signer_credentials)?;
                memo_builder.set_payment_intent_id(*payment_intent_id);
                Ok(Box::new(memo_builder))
            }
            Self::RTHWithPaymentRequestId {
                subaddress_index,
                payment_request_id,
            } => {
                let mut memo_builder =
                    generate_rth_memo_builder(subaddress_index, signer_credentials)?;
                memo_builder.set_payment_request_id(*payment_request_id);
                Ok(Box::new(memo_builder))
            }
        }
    }
}

fn generate_rth_memo_builder(
    subaddress_index: &Option<u64>,
    signer_credentials: &TransactionMemoSignerCredentials,
) -> Result<RTHMemoBuilder, WalletTransactionBuilderError> {
    let subaddress_index = subaddress_index.unwrap_or(DEFAULT_SUBADDRESS_INDEX);
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
                    &account_key.subaddress(subaddress_index),
                    account_key.subaddress_spend_private(subaddress_index),
                ),
            );
        }

        TransactionMemoSignerCredentials::HardwareWallet(view_account_key) => {
            let signer: Arc<Box<dyn AuthenticatedMemoHmacSigner + 'static + Send + Sync>> =
                Arc::new(Box::new(HardwareWalletAuthenticatedMemoHmacSigner::new(
                    &view_account_key.subaddress(subaddress_index),
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
