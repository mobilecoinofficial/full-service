use std::convert::TryInto;

use mc_account_keys::PublicAddress;
use mc_crypto_keys::{GenericArray, ReprBytes, RistrettoPublic};
use mc_transaction_core::{
    ring_signature::KeyImage,
    tx::{Tx, TxOut},
    TokenId,
};

use crate::util::b58::b58_decode_public_address;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputTxo {
    pub tx_out: TxOut,
    pub key_image: KeyImage,
    pub value: u64,
    pub token_id: TokenId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutputTxo {
    pub tx_out: TxOut,
    pub recipient_public_address: PublicAddress,
    pub shared_secret: RistrettoPublic,
    pub value: u64,
    pub token_id: TokenId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TxProposal {
    pub tx: Tx,
    pub input_txos: Vec<InputTxo>,
    pub payload_txos: Vec<OutputTxo>,
    pub change_txos: Vec<OutputTxo>,
}

impl From<&crate::json_rpc::tx_proposal::TxProposalJSON> for TxProposal {
    fn from(src: &crate::json_rpc::tx_proposal::TxProposalJSON) -> Self {
        let tx = mc_util_serial::decode(hex::decode(&src.tx_proto).unwrap().as_slice()).unwrap();
        let input_txos = src
            .input_txos
            .iter()
            .map(|input_txo| {
                let key_image_bytes: [u8; 32] = hex::decode(&input_txo.key_image)
                    .unwrap()
                    .as_slice()
                    .try_into()
                    .unwrap();
                InputTxo {
                    tx_out: mc_util_serial::decode(
                        hex::decode(&input_txo.tx_out_proto).unwrap().as_slice(),
                    )
                    .unwrap(),
                    key_image: KeyImage::from(key_image_bytes),
                    value: input_txo.value.parse::<u64>().unwrap(),
                    token_id: TokenId::from(input_txo.token_id.parse::<u64>().unwrap()),
                }
            })
            .collect();

        let payload_txos = src
            .payload_txos
            .iter()
            .map(|payload_txo| {
                let shared_secret_bytes: [u8; 32] = hex::decode(&payload_txo.shared_secret)
                    .unwrap()
                    .as_slice()
                    .try_into()
                    .unwrap();
                OutputTxo {
                    tx_out: mc_util_serial::decode(
                        hex::decode(&payload_txo.tx_out_proto).unwrap().as_slice(),
                    )
                    .unwrap(),
                    recipient_public_address: b58_decode_public_address(
                        &payload_txo.recipient_public_address_b58,
                    )
                    .unwrap(),
                    shared_secret: RistrettoPublic::from_bytes(GenericArray::from_slice(
                        &shared_secret_bytes,
                    ))
                    .unwrap(),
                    value: payload_txo.value.parse::<u64>().unwrap(),
                    token_id: TokenId::from(payload_txo.token_id.parse::<u64>().unwrap()),
                }
            })
            .collect();

        let change_txos = src
            .change_txos
            .iter()
            .map(|change_txo| {
                let shared_secret_bytes: [u8; 32] = hex::decode(&change_txo.shared_secret)
                    .unwrap()
                    .as_slice()
                    .try_into()
                    .unwrap();
                OutputTxo {
                    tx_out: mc_util_serial::decode(
                        hex::decode(&change_txo.tx_out_proto).unwrap().as_slice(),
                    )
                    .unwrap(),
                    recipient_public_address: b58_decode_public_address(
                        &change_txo.recipient_public_address_b58,
                    )
                    .unwrap(),
                    shared_secret: RistrettoPublic::from_bytes(GenericArray::from_slice(
                        &shared_secret_bytes,
                    ))
                    .unwrap(),
                    value: change_txo.value.parse::<u64>().unwrap(),
                    token_id: TokenId::from(change_txo.token_id.parse::<u64>().unwrap()),
                }
            })
            .collect();

        Self {
            tx,
            input_txos,
            payload_txos,
            change_txos,
        }
    }
}
