use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Memo {
    Unused,
    AuthenticatedSender(AuthenticatedSenderMemo),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AuthenticatedSenderMemo {
    pub sender_address_hash: String,
    pub payment_request_id: Option<String>,
    pub payment_intent_id: Option<String>,
}
