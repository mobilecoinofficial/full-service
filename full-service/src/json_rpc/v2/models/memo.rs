use crate::db::{models::AuthenticatedSenderMemo, txo::TxoMemo};
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub enum Memo {
    #[default]
    Unused,
    Sender(SenderMemo),
}

/// This represents data that is included in any of:
/// * AuthenticatedSenderMemo,
/// * AuthenticatedSenderMemoWithPaymentRequest
/// * AuthenticatedSenderMemoWithPaymentIntent
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SenderMemo {
    pub sender_address_hash: String,
    pub payment_request_id: Option<String>,
    pub payment_intent_id: Option<String>,
}

impl From<&AuthenticatedSenderMemo> for SenderMemo {
    fn from(memo: &AuthenticatedSenderMemo) -> Self {
        SenderMemo {
            sender_address_hash: memo.sender_address_hash.clone(),
            payment_request_id: memo.payment_request_id.clone(),
            payment_intent_id: memo.payment_intent_id.clone(),
        }
    }
}

impl From<&TxoMemo> for Memo {
    fn from(memo: &TxoMemo) -> Self {
        match memo {
            TxoMemo::AuthenticatedSender(memo) => Memo::Sender(memo.into()),
            _ => Memo::Unused,
        }
    }
}
