// Copyright (c) 2020-2023 MobileCoin Inc.

use crate::db::{models::AuthenticatedSenderMemo as AuthenticatedSenderMemoDbModel, txo::TxoMemo};
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub enum Memo {
    #[default]
    Unused,
    AuthenticatedSender(AuthenticatedSenderMemo),
}

/// This represents data that is included in any of:
/// * AuthenticatedSenderMemo,
/// * AuthenticatedSenderMemoWithPaymentRequest
/// * AuthenticatedSenderMemoWithPaymentIntent
#[derive(Deserialize, Serialize, Debug, Clone)]
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

impl From<&TxoMemo> for Memo {
    fn from(memo: &TxoMemo) -> Self {
        match memo {
            TxoMemo::AuthenticatedSender(memo) => Memo::AuthenticatedSender(memo.into()),
            TxoMemo::Unused => Memo::Unused,
        }
    }
}
