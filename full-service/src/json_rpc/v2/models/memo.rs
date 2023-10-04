use crate::db::models::AuthenticatedSenderMemo;
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Memo {
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

impl From<AuthenticatedSenderMemo> for SenderMemo {
    fn from(memo: AuthenticatedSenderMemo) -> Self {
        SenderMemo {
            sender_address_hash: memo.sender_address_hash,
            payment_request_id: memo.payment_request_id,
            payment_intent_id: memo.payment_intent_id,
        }
    }
}
