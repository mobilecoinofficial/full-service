// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the TxProposal object.

use crate::json_rpc::unspent_tx_out::UnspentTxOut;
use mc_mobilecoind_json::data_types::{JsonOutlay, JsonTx, JsonUnspentTxOut};

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

impl From<&mc_mobilecoind::payments::TxProposal> for TxProposal {
    fn from(src: &mc_mobilecoind::payments::TxProposal) -> Self {
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
        Self {
            input_list: json_tx_proposal
                .input_list
                .iter()
                .map(UnspentTxOut::from)
                .collect(),
            outlay_list: json_tx_proposal.outlay_list.clone(),
            tx: json_tx_proposal.tx.clone(),
            fee: json_tx_proposal.fee.to_string(),
            outlay_index_to_tx_out_index: outlay_map,
            outlay_confirmation_numbers: json_tx_proposal.outlay_confirmation_numbers.clone(),
        }
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
