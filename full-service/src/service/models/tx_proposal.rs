use std::convert::{TryFrom, TryInto};

use mc_account_keys::PublicAddress;
use mc_transaction_core::{
    ring_signature::KeyImage,
    tokens::Mob,
    tx::{Tx, TxOut, TxOutConfirmationNumber},
    Amount, Token, TokenId,
};

use crate::util::b58::b58_decode_public_address;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputTxo {
    pub tx_out: TxOut,
    pub subaddress_index: u64,
    pub key_image: KeyImage,
    pub amount: Amount,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutputTxo {
    pub tx_out: TxOut,
    pub recipient_public_address: PublicAddress,
    pub confirmation_number: TxOutConfirmationNumber,
    pub amount: Amount,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TxProposal {
    pub tx: Tx,
    pub input_txos: Vec<InputTxo>,
    pub payload_txos: Vec<OutputTxo>,
    pub change_txos: Vec<OutputTxo>,
}

impl From<&crate::json_rpc::v1::models::tx_proposal::TxProposal> for TxProposal {
    fn from(src: &crate::json_rpc::v1::models::tx_proposal::TxProposal) -> Self {
        let mc_api_tx = mc_api::external::Tx::try_from(&src.tx).unwrap();
        let tx = Tx::try_from(&mc_api_tx).unwrap();

        let input_txos = src
            .input_list
            .iter()
            .map(|unspent_txo| {
                let mc_api_tx_out = mc_api::external::TxOut::try_from(&unspent_txo.tx_out).unwrap();
                let tx_out = TxOut::try_from(&mc_api_tx_out).unwrap();

                let key_image_bytes = hex::decode(unspent_txo.key_image.clone()).unwrap();
                let key_image = KeyImage::try_from(key_image_bytes.as_slice()).unwrap();

                InputTxo {
                    tx_out,
                    subaddress_index: unspent_txo.subaddress_index.parse::<u64>().unwrap(),
                    key_image,
                    amount: Amount::new(unspent_txo.value, Mob::ID),
                }
            })
            .collect();

        let mut payload_txos = Vec::new();

        for (outlay_index, tx_out_index) in src.outlay_index_to_tx_out_index.iter() {
            let outlay_index = outlay_index.parse::<usize>().unwrap();
            let outlay = &src.outlay_list[outlay_index];
            let tx_out_index = tx_out_index.parse::<usize>().unwrap();
            let tx_out = tx.prefix.outputs[tx_out_index].clone();
            let confirmation_number_bytes: [u8; 32] = src.outlay_confirmation_numbers[outlay_index]
                .clone()
                .try_into()
                .unwrap();

            let confirmation_number = TxOutConfirmationNumber::from(confirmation_number_bytes);

            let mc_api_public_address =
                mc_api::external::PublicAddress::try_from(&outlay.receiver).unwrap();
            let public_address = PublicAddress::try_from(&mc_api_public_address).unwrap();

            let payload_txo = OutputTxo {
                tx_out,
                recipient_public_address: public_address,
                confirmation_number,
                amount: Amount::new(outlay.value.0, Mob::ID),
            };

            payload_txos.push(payload_txo);
        }

        Self {
            tx,
            input_txos,
            payload_txos,
            change_txos: Vec::new(),
        }
    }
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
                    amount: Amount::try_from(&input_txo.amount).unwrap(),
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
                    amount: Amount::try_from(&payload_txo.amount).unwrap(),
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
                    amount: Amount::try_from(&change_txo.amount).unwrap(),
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
