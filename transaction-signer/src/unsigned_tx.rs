use mc_crypto_keys::RistrettoPublic;
use mc_transaction_core::{
    ring_signature::KeyImage,
    tx::{TxIn, TxOut},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct UnsignedTx {
    /// List of input rings for the transaction, where each ring contains a
    /// single real input that is associated with the corresponding KeyImage
    pub inputs_and_key_images: Vec<(TxIn, KeyImage)>,

    /// List of outputs and shared secrets for the transaction
    pub outputs_and_shared_secrets: Vec<(TxOut, RistrettoPublic)>,

    /// Fee paid to the foundation for this transaction
    pub fee: u64,
}
