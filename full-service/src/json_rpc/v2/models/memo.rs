// Copyright (c) 2020-2023 MobileCoin Inc.

use crate::db::{
    models::{
        AuthenticatedSenderMemo as AuthenticatedSenderMemoDbModel,
        DestinationMemo as DestinationMemoDbModel,
    },
    txo::TxoMemo,
};
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Debug, Clone, PartialEq)]
pub enum Memo {
    #[default]
    Unused,
    AuthenticatedSender(AuthenticatedSenderMemo),
    Destination(DestinationMemo),
}

/// This represents data that is included in any of:
/// * AuthenticatedSenderMemo,
/// * AuthenticatedSenderMemoWithPaymentRequest
/// * AuthenticatedSenderMemoWithPaymentIntent
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct AuthenticatedSenderMemo {
    pub sender_address_hash: String,
    pub payment_request_id: Option<String>,
    pub payment_intent_id: Option<String>,
}

impl From<&AuthenticatedSenderMemoDbModel> for AuthenticatedSenderMemo {
    fn from(memo: &AuthenticatedSenderMemoDbModel) -> Self {
        AuthenticatedSenderMemo {
            sender_address_hash: memo.sender_address_hash.clone(),
            payment_request_id: memo.payment_request_id.map(|id| id.to_string()),
            payment_intent_id: memo.payment_intent_id.map(|id| id.to_string()),
        }
    }
}

/// This represents data that is included in any of:
/// * DestinationMemo,
/// * DestinationWithPaymentRequest
/// * DestinationWithPaymentIntent
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct DestinationMemo {
    pub recipient_address_hash: String,
    pub num_recipients: String,
    pub fee: String,
    pub total_outlay: String,
    pub payment_request_id: Option<String>,
    pub payment_intent_id: Option<String>,
}

impl From<&DestinationMemoDbModel> for DestinationMemo {
    fn from(memo: &DestinationMemoDbModel) -> Self {
        DestinationMemo {
            recipient_address_hash: memo.recipient_address_hash.clone(),
            num_recipients: memo.num_recipients.to_string(),
            fee: memo.fee.to_string(),
            total_outlay: memo.total_outlay.to_string(),
            payment_request_id: memo.payment_request_id.map(|id| id.to_string()),
            payment_intent_id: memo.payment_intent_id.map(|id| id.to_string()),
        }
    }
}

impl From<&TxoMemo> for Memo {
    fn from(memo: &TxoMemo) -> Self {
        match memo {
            TxoMemo::AuthenticatedSender(memo) => Memo::AuthenticatedSender(memo.into()),
            TxoMemo::Destination(memo) => Memo::Destination(memo.into()),
            TxoMemo::Unused => Memo::Unused,
        }
    }
}
