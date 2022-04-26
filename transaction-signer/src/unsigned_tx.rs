use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Eq, Hash, PartialEq, Serialize, Message, Digestible)]
pub struct UnsignedTx {
    /// List of possible inputs for the transaction, where each owned input will
    /// be included in the constructed transaction and each unowned input
    /// will be used as a ring input during ring construction
    #[prost(message, repeated, tag = "1")]
    pub inputs: Vec<TxOut>,

    /// List of outputs for the transaction
    #[prost(message, repeated, tag = "2")]
    pub outputs: Vec<TxOut>,

    /// Fee paid to the foundation for this transaction
    #[prost(uint64, tag = "3")]
    pub fee: u64,
}
