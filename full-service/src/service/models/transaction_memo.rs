// Copyright (c) 2020-2025 MobileCoin Inc.

use crate::{
    db::{account::AccountModel, models::Account},
    error::WalletTransactionBuilderError,
    service::hardware_wallet::HardwareWalletAuthenticatedMemoHmacSigner,
};
use mc_account_keys::DEFAULT_SUBADDRESS_INDEX;
use mc_transaction_builder::{
    BurnRedemptionMemoBuilder, EmptyMemoBuilder, MemoBuilder, RTHMemoBuilder,
};
use mc_transaction_extra::{BurnRedemptionMemo, SenderMemoCredential};
use mc_util_serial::BigArray;
use serde::{Deserialize, Serialize};
use std::{boxed::Box, sync::Arc};

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
        account: &Account,
    ) -> Result<Box<dyn MemoBuilder + Send + Sync>, WalletTransactionBuilderError> {
        match self {
            Self::Empty => Ok(Box::<EmptyMemoBuilder>::default()),
            Self::BurnRedemption(memo_data) => {
                let mut memo_builder = BurnRedemptionMemoBuilder::new(*memo_data);
                memo_builder.enable_destination_memo();
                Ok(Box::new(memo_builder))
            }
            Self::RTH { subaddress_index } => {
                let memo_builder = generate_rth_memo_builder(subaddress_index, account)?;
                Ok(Box::new(memo_builder))
            }
            Self::RTHWithPaymentIntentId {
                subaddress_index,
                payment_intent_id,
            } => {
                let mut memo_builder = generate_rth_memo_builder(subaddress_index, account)?;
                memo_builder.set_payment_intent_id(*payment_intent_id);
                Ok(Box::new(memo_builder))
            }
            Self::RTHWithPaymentRequestId {
                subaddress_index,
                payment_request_id,
            } => {
                let mut memo_builder = generate_rth_memo_builder(subaddress_index, account)?;
                memo_builder.set_payment_request_id(*payment_request_id);
                Ok(Box::new(memo_builder))
            }
        }
    }
}

fn generate_rth_memo_builder(
    subaddress_index: &Option<u64>,
    account: &Account,
) -> Result<RTHMemoBuilder, WalletTransactionBuilderError> {
    let subaddress_index = subaddress_index.unwrap_or(DEFAULT_SUBADDRESS_INDEX);
    let mut memo_builder = RTHMemoBuilder::default();

    if account.view_only {
        if !account.managed_by_hardware_wallet {
            return Err(WalletTransactionBuilderError::RTHUnavailableForViewOnlyAccounts);
        }

        let view_account_key = account.view_account_key()?;
        memo_builder.set_authenticated_sender_hmac_signer(Arc::new(Box::new(
            HardwareWalletAuthenticatedMemoHmacSigner::new(
                &view_account_key.subaddress(subaddress_index),
                subaddress_index,
            )?,
        )));
    } else {
        let account_key = account.account_key()?;
        memo_builder.set_sender_credential(
            SenderMemoCredential::new_from_address_and_spend_private_key(
                &account_key.subaddress(subaddress_index),
                account_key.subaddress_spend_private(subaddress_index),
            ),
        );
    }

    memo_builder.enable_destination_memo();

    Ok(memo_builder)
}
