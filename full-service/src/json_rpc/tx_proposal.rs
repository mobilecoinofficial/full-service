// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the TxProposal object.

use crate::util::b58::{b58_encode_public_address, B58Error};

use serde_derive::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct InputTxo {
    pub tx_out_proto: String,
    pub value: String,
    pub token_id: String,
    pub key_image: String,
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct OutputTxo {
    pub tx_out_proto: String,
    pub value: String,
    pub token_id: String,
    pub recipient_public_address_b58: String,
    pub confirmation_number: String,
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct TxProposal {
    pub input_txos: Vec<InputTxo>,
    pub payload_txos: Vec<OutputTxo>,
    pub change_txos: Vec<OutputTxo>,
    pub fee: String,
    pub fee_token_id: String,
    pub tombstone_block_index: String,
    pub tx_proto: String,
}

impl TryFrom<&crate::service::models::tx_proposal::TxProposal> for TxProposal {
    type Error = String;

    fn try_from(src: &crate::service::models::tx_proposal::TxProposal) -> Result<Self, String> {
        let input_txos = src
            .input_txos
            .iter()
            .map(|input_txo| InputTxo {
                tx_out_proto: hex::encode(mc_util_serial::encode(&input_txo.tx_out)),
                value: input_txo.value.to_string(),
                token_id: input_txo.token_id.to_string(),
                key_image: hex::encode(&input_txo.key_image.as_bytes()),
            })
            .collect();

        let payload_txos = src
            .payload_txos
            .iter()
            .map(|output_txo| {
                Ok(OutputTxo {
                    tx_out_proto: hex::encode(mc_util_serial::encode(&output_txo.tx_out)),
                    value: output_txo.value.to_string(),
                    token_id: output_txo.token_id.to_string(),
                    recipient_public_address_b58: b58_encode_public_address(
                        &output_txo.recipient_public_address,
                    )?,
                    confirmation_number: hex::encode(output_txo.confirmation_number.as_ref()),
                })
            })
            .collect::<Result<Vec<OutputTxo>, B58Error>>()
            .map_err(|_| "Error".to_string())?;

        let change_txos = src
            .change_txos
            .iter()
            .map(|output_txo| {
                Ok(OutputTxo {
                    tx_out_proto: hex::encode(mc_util_serial::encode(&output_txo.tx_out)),
                    value: output_txo.value.to_string(),
                    token_id: output_txo.token_id.to_string(),
                    recipient_public_address_b58: b58_encode_public_address(
                        &output_txo.recipient_public_address,
                    )?,
                    confirmation_number: hex::encode(output_txo.confirmation_number.as_ref()),
                })
            })
            .collect::<Result<Vec<OutputTxo>, B58Error>>()
            .map_err(|_| "Error".to_string())?;

        Ok(Self {
            input_txos,
            payload_txos,
            change_txos,
            tx_proto: hex::encode(mc_util_serial::encode(&src.tx)),
            fee: src.tx.prefix.fee.to_string(),
            fee_token_id: src.tx.prefix.fee_token_id.to_string(),
            tombstone_block_index: src.tx.prefix.tombstone_block.to_string(),
        })
    }
}
