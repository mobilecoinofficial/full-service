// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the TxProposal object.

use crate::{
    json_rpc::unspent_tx_out::UnspentTxOut,
    util::b58::{b58_encode_public_address, B58Error},
};
use mc_mobilecoind_json::data_types::{JsonOutlay, JsonTx, JsonTxOut, JsonUnspentTxOut};

use serde_derive::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct TxProposal {
    pub input_list: Vec<UnspentTxOut>,
    pub outlay_list: Vec<JsonOutlay>,
    pub tx: JsonTx,
    pub fee: String,
    pub outlay_index_to_tx_out_index: Vec<(String, String)>,
    pub outlay_confirmation_numbers: Vec<Vec<u8>>,
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct InputTxoJSON {
    pub tx_out_proto: String,
    pub value: String,
    pub token_id: String,
    pub key_image: String,
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct OutputTxoJSON {
    pub tx_out_proto: String,
    pub value: String,
    pub token_id: String,
    pub recipient_public_address_b58: String,
    pub confirmation_number: String,
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct TxProposalJSON {
    pub input_txos: Vec<InputTxoJSON>,
    pub payload_txos: Vec<OutputTxoJSON>,
    pub change_txos: Vec<OutputTxoJSON>,
    pub fee: String,
    pub fee_token_id: String,
    pub tombstone_block_index: String,
    pub tx_proto: String,
}

impl TryFrom<&crate::service::models::tx_proposal::TxProposal> for TxProposalJSON {
    type Error = String;

    fn try_from(src: &crate::service::models::tx_proposal::TxProposal) -> Result<Self, String> {
        let input_txos = src
            .input_txos
            .iter()
            .map(|input_txo| InputTxoJSON {
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
                Ok(OutputTxoJSON {
                    tx_out_proto: hex::encode(mc_util_serial::encode(&output_txo.tx_out)),
                    value: output_txo.value.to_string(),
                    token_id: output_txo.token_id.to_string(),
                    recipient_public_address_b58: b58_encode_public_address(
                        &output_txo.recipient_public_address,
                    )?,
                    confirmation_number: hex::encode(output_txo.confirmation_number.as_ref()),
                })
            })
            .collect::<Result<Vec<OutputTxoJSON>, B58Error>>()
            .map_err(|_| "Error".to_string())?;

        let change_txos = src
            .change_txos
            .iter()
            .map(|output_txo| {
                Ok(OutputTxoJSON {
                    tx_out_proto: hex::encode(mc_util_serial::encode(&output_txo.tx_out)),
                    value: output_txo.value.to_string(),
                    token_id: output_txo.token_id.to_string(),
                    recipient_public_address_b58: b58_encode_public_address(
                        &output_txo.recipient_public_address,
                    )?,
                    confirmation_number: hex::encode(output_txo.confirmation_number.as_ref()),
                })
            })
            .collect::<Result<Vec<OutputTxoJSON>, B58Error>>()
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

impl TryFrom<&mc_mobilecoind::payments::TxProposal> for TxProposal {
    type Error = String;

    fn try_from(src: &mc_mobilecoind::payments::TxProposal) -> Result<Self, String> {
        // FIXME: WS-34 - Several unnecessary conversions, but we're leveraging existing
        // conversion code.

        // First, convert it to the proto
        let proto_tx_proposal = mc_mobilecoind_api::TxProposal::from(src);

        // Then, convert it to the json representation
        let json_tx_proposal =
            mc_mobilecoind_json::data_types::JsonTxProposal::from(&proto_tx_proposal);

        let outlay_map: Vec<(String, String)> = json_tx_proposal
            .outlay_index_to_tx_out_index
            .iter()
            .map(|(key, val)| (key.to_string(), val.to_string()))
            .collect();
        Ok(Self {
            input_list: json_tx_proposal
                .input_list
                .iter()
                .map(UnspentTxOut::try_from)
                .collect::<Result<Vec<UnspentTxOut>, String>>()?,
            outlay_list: json_tx_proposal.outlay_list.clone(),
            tx: json_tx_proposal.tx.clone(),
            fee: json_tx_proposal.fee.to_string(),
            outlay_index_to_tx_out_index: outlay_map,
            outlay_confirmation_numbers: json_tx_proposal.outlay_confirmation_numbers.clone(),
        })
    }
}

impl TryFrom<&TxProposal> for mc_mobilecoind::payments::TxProposal {
    type Error = String;

    #[allow(clippy::bind_instead_of_map)]
    fn try_from(src: &TxProposal) -> Result<mc_mobilecoind::payments::TxProposal, String> {
        // First, convert to the JsonTxProposal
        let json_tx_proposal = mc_mobilecoind_json::data_types::JsonTxProposal::try_from(src)
            .map_err(|err| format!("Failed to parse tx_proposal from json_rpc type {:?}", err))?;

        // Then convert to the proto tx proposal
        let proto_tx_proposal = mc_mobilecoind_api::TxProposal::try_from(&json_tx_proposal)
            .map_err(|err| format!("Failed to parse tx_proposal from json: {:?}", err))?;

        // Last, convert to the mobilecoind type
        let tx_proposal = mc_mobilecoind::payments::TxProposal::try_from(&proto_tx_proposal)
            .map_err(|err| format!("Failed to parse tx_proposal from proto: {:?}", err))?;
        Ok(tx_proposal)
    }
}

// FIXME: remove below
impl TryFrom<&TxProposal> for mc_mobilecoind_json::data_types::JsonTxProposal {
    type Error = String;

    #[allow(clippy::bind_instead_of_map)]
    fn try_from(
        src: &TxProposal,
    ) -> Result<mc_mobilecoind_json::data_types::JsonTxProposal, String> {
        let outlay_map: Vec<(usize, usize)> = src
            .outlay_index_to_tx_out_index
            .iter()
            .map(|(key, val)| {
                key.parse::<usize>()
                    .and_then(|k| val.parse::<usize>().and_then(|v| Ok((k, v))))
                    .map_err(|err| format!("Failed to parse u64 from outlay_map: {}", err))
            })
            .collect::<Result<Vec<(usize, usize)>, String>>()?;
        Ok(Self {
            input_list: src
                .input_list
                .iter()
                .map(JsonUnspentTxOut::try_from)
                .collect::<Result<Vec<JsonUnspentTxOut>, String>>()?,
            outlay_list: src.outlay_list.clone(),
            tx: src.tx.clone(),
            fee: src
                .fee
                .parse::<u64>()
                .map_err(|err| format!("Failed to parse u64 from fee: {}", err))?,
            outlay_index_to_tx_out_index: outlay_map,
            outlay_confirmation_numbers: src.outlay_confirmation_numbers.clone(),
        })
    }
}
