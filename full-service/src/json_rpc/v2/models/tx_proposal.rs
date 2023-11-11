// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the TxProposal object.

use super::amount::Amount as AmountJSON;
use crate::util::b58::{b58_encode_public_address, B58Error};

use protobuf::Message;
use redact::{expose_secret, Secret};
use serde_derive::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Deserialize, Serialize, Default, Debug, PartialEq)]
pub struct UnsignedInputTxo {
    pub tx_out_proto: String,
    pub amount: AmountJSON,
    pub subaddress_index: String,
}

#[derive(Clone, Deserialize, Serialize, Default, Debug, PartialEq)]
pub struct InputTxo {
    pub tx_out_proto: String,
    pub amount: AmountJSON,
    pub subaddress_index: String,
    #[serde(serialize_with = "expose_secret")]
    pub key_image: Secret<String>,
}

#[derive(Clone, Deserialize, Serialize, Default, Debug, PartialEq)]
pub struct OutputTxo {
    pub tx_out_proto: String,
    pub amount: AmountJSON,
    pub recipient_public_address_b58: String,
    pub confirmation_number: String,
    pub shared_secret: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub struct UnsignedTxProposal {
    pub unsigned_tx_proto_bytes_hex: String,
    pub unsigned_input_txos: Vec<UnsignedInputTxo>,
    pub payload_txos: Vec<OutputTxo>,
    pub change_txos: Vec<OutputTxo>,
}

impl TryFrom<&crate::service::models::tx_proposal::UnsignedTxProposal> for UnsignedTxProposal {
    type Error = String;

    fn try_from(
        src: &crate::service::models::tx_proposal::UnsignedTxProposal,
    ) -> Result<Self, Self::Error> {
        let unsigned_input_txos = src
            .unsigned_input_txos
            .iter()
            .map(|input_txo| UnsignedInputTxo {
                tx_out_proto: hex::encode(mc_util_serial::encode(&input_txo.tx_out)),
                amount: AmountJSON::from(&input_txo.amount),
                subaddress_index: input_txo.subaddress_index.to_string(),
            })
            .collect();

        let payload_txos = src
            .payload_txos
            .iter()
            .map(|output_txo| {
                Ok(OutputTxo {
                    tx_out_proto: hex::encode(mc_util_serial::encode(&output_txo.tx_out)),
                    amount: AmountJSON::from(&output_txo.amount),
                    recipient_public_address_b58: b58_encode_public_address(
                        &output_txo.recipient_public_address,
                    )?,
                    confirmation_number: hex::encode(output_txo.confirmation_number.as_ref()),
                    shared_secret: output_txo
                        .shared_secret
                        .map(|shared_secret| hex::encode(shared_secret.to_bytes())),
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
                    amount: AmountJSON::from(&output_txo.amount),
                    recipient_public_address_b58: b58_encode_public_address(
                        &output_txo.recipient_public_address,
                    )?,
                    confirmation_number: hex::encode(output_txo.confirmation_number.as_ref()),
                    shared_secret: output_txo
                        .shared_secret
                        .map(|shared_secret| hex::encode(shared_secret.to_bytes())),
                })
            })
            .collect::<Result<Vec<OutputTxo>, B58Error>>()
            .map_err(|_| "Error".to_string())?;

        let unsigned_tx_external: mc_api::external::UnsignedTx = (&src.unsigned_tx).into();
        let unsigned_tx_proto_bytes = unsigned_tx_external
            .write_to_bytes()
            .map_err(|e| e.to_string())?;
        let unsigned_tx_proto_bytes_hex = hex::encode(unsigned_tx_proto_bytes.as_slice());

        Ok(Self {
            unsigned_tx_proto_bytes_hex,
            unsigned_input_txos,
            payload_txos,
            change_txos,
        })
    }
}

#[derive(Clone, Deserialize, Serialize, Default, Debug, PartialEq)]
pub struct TxProposal {
    pub input_txos: Vec<InputTxo>,
    pub payload_txos: Vec<OutputTxo>,
    pub change_txos: Vec<OutputTxo>,
    pub fee_amount: AmountJSON,
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
                amount: AmountJSON::from(&input_txo.amount),
                subaddress_index: input_txo.subaddress_index.to_string(),
                key_image: hex::encode(input_txo.key_image.as_bytes()).into(),
            })
            .collect();

        let payload_txos = src
            .payload_txos
            .iter()
            .map(|output_txo| {
                Ok(OutputTxo {
                    tx_out_proto: hex::encode(mc_util_serial::encode(&output_txo.tx_out)),
                    amount: AmountJSON::from(&output_txo.amount),
                    recipient_public_address_b58: b58_encode_public_address(
                        &output_txo.recipient_public_address,
                    )?,
                    confirmation_number: hex::encode(output_txo.confirmation_number.as_ref()),
                    shared_secret: output_txo
                        .shared_secret
                        .map(|shared_secret| hex::encode(shared_secret.to_bytes())),
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
                    amount: AmountJSON::from(&output_txo.amount),
                    recipient_public_address_b58: b58_encode_public_address(
                        &output_txo.recipient_public_address,
                    )?,
                    confirmation_number: hex::encode(output_txo.confirmation_number.as_ref()),
                    shared_secret: output_txo
                        .shared_secret
                        .map(|shared_secret| hex::encode(shared_secret.to_bytes())),
                })
            })
            .collect::<Result<Vec<OutputTxo>, B58Error>>()
            .map_err(|_| "Error".to_string())?;

        Ok(Self {
            input_txos,
            payload_txos,
            change_txos,
            tx_proto: hex::encode(mc_util_serial::encode(&src.tx)),
            fee_amount: AmountJSON::new(src.tx.prefix.fee, src.tx.prefix.fee_token_id.into()),
            tombstone_block_index: src.tx.prefix.tombstone_block.to_string(),
        })
    }
}
