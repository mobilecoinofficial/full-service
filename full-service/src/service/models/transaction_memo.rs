// Copyright (c) 2020-2025 MobileCoin Inc.

use mc_account_keys::AccountKey;
use mc_transaction_builder::{
    BurnRedemptionMemoBuilder, EmptyMemoBuilder, MemoBuilder, RTHMemoBuilder,
};
use mc_transaction_extra::{BurnRedemptionMemo, SenderMemoCredential};
use mc_util_serial::BigArray;
use serde::{Deserialize, Serialize};

/// This represents the different types of Transaction Memos that can be used in
/// a given transaction
///
/// * Empty
///
/// * RTH
///
/// * BurnRedemption
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum TransactionMemo {
    /// Empty Transaction Memo.
    Empty,

    /// Recoverable Transaction History memo with an optional u64 specifying the
    /// subaddress index to generate the sender memo credential from
    RTH {
        /// Optional subaddress index to generate the sender memo credential
        /// from.
        subaddress_index: Option<u64>,
    },

    RTHWithPaymentIntentId {
        /// Optional subaddress index to generate the sender memo credential
        /// from.
        subaddress_index: Option<u64>,

        /// The payment intent id to include in the memo.
        payment_intent_id: u64,
    },

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
    pub fn memo_builder(&self, account_key: &AccountKey) -> Box<dyn MemoBuilder + Send + Sync> {
        match self {
            Self::Empty => Box::<EmptyMemoBuilder>::default(),
            Self::RTH { subaddress_index } => {
                let memo_builder = generate_rth_memo_builder(subaddress_index, account_key);
                Box::new(memo_builder)
            }
            Self::RTHWithPaymentIntentId {
                subaddress_index,
                payment_intent_id,
            } => {
                let mut memo_builder = generate_rth_memo_builder(subaddress_index, account_key);
                memo_builder.set_payment_intent_id(*payment_intent_id);
                Box::new(memo_builder)
            }
            Self::RTHWithPaymentRequestId {
                subaddress_index,
                payment_request_id,
            } => {
                let mut memo_builder = generate_rth_memo_builder(subaddress_index, account_key);
                memo_builder.set_payment_request_id(*payment_request_id);
                Box::new(memo_builder)
            }
            Self::BurnRedemption(memo_data) => {
                let mut memo_builder = BurnRedemptionMemoBuilder::new(*memo_data);
                memo_builder.enable_destination_memo();
                Box::new(memo_builder)
            }
        }
    }
}

fn generate_rth_memo_builder(
    subaddress_index: &Option<u64>,
    account_key: &AccountKey,
) -> RTHMemoBuilder {
    let mut memo_builder = RTHMemoBuilder::default();
    let sender_memo_credential = match subaddress_index {
        Some(subaddress_index) => SenderMemoCredential::new_from_address_and_spend_private_key(
            &account_key.subaddress(*subaddress_index),
            account_key.subaddress_spend_private(*subaddress_index),
        ),
        None => SenderMemoCredential::from(account_key),
    };
    memo_builder.set_sender_credential(sender_memo_credential);
    memo_builder.enable_destination_memo();

    memo_builder
}
