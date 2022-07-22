use std::convert::TryInto;

use mc_account_keys::PublicAddress;
use mc_transaction_core::{
    ring_signature::KeyImage,
    tx::{Tx, TxOut, TxOutConfirmationNumber},
    TokenId,
};

use crate::util::b58::b58_decode_public_address;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputTxo {
    pub tx_out: TxOut,
    pub subaddress_index: u64,
    pub key_image: KeyImage,
    pub value: u64,
    pub token_id: TokenId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutputTxo {
    pub tx_out: TxOut,
    pub recipient_public_address: PublicAddress,
    pub confirmation_number: TxOutConfirmationNumber,
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

impl From<&crate::json_rpc::v2::models::tx_proposal::TxProposal> for TxProposal {
    fn from(src: &crate::json_rpc::v2::models::tx_proposal::TxProposal) -> Self {
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
                    subaddress_index: input_txo.subaddress_index.parse::<u64>().unwrap(),
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
                let confirmation_number_bytes: [u8; 32] =
                    hex::decode(&payload_txo.confirmation_number)
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
                    confirmation_number: TxOutConfirmationNumber::from(&confirmation_number_bytes),
                    value: payload_txo.value.parse::<u64>().unwrap(),
                    token_id: TokenId::from(payload_txo.token_id.parse::<u64>().unwrap()),
                }
            })
            .collect();

        let change_txos = src
            .change_txos
            .iter()
            .map(|change_txo| {
                let confirmation_number_bytes: [u8; 32] =
                    hex::decode(&change_txo.confirmation_number)
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
                    confirmation_number: TxOutConfirmationNumber::from(&confirmation_number_bytes),
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
