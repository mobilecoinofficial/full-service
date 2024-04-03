// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the TxProposal object.

use crate::{
    json_rpc::v1::models::unspent_tx_out::UnspentTxOut,
    service::models::tx_proposal::TxProposal as TxProposalServiceModel,
};
use mc_common::HashMap;
use mc_mobilecoind_json::data_types::{JsonOutlayV2, JsonTx, JsonUnspentTxOut};

use mc_transaction_core::MemoPayload;
use mc_transaction_extra::TxOutConfirmationNumber;
use redact::{expose_secret, Secret};
use serde_derive::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct TxProposal {
    pub input_list: Vec<UnspentTxOut>,
    pub outlay_list: Vec<JsonOutlayV2>,
    pub tx: JsonTx,
    #[serde(serialize_with = "expose_secret")]
    pub fee: Secret<String>,
    #[serde(serialize_with = "expose_secret")]
    pub outlay_index_to_tx_out_index: Secret<Vec<(String, String)>>,
    #[serde(serialize_with = "expose_secret")]
    pub outlay_confirmation_numbers: Secret<Vec<Vec<u8>>>,
}

impl TryFrom<&TxProposalServiceModel> for TxProposal {
    type Error = String;

    fn try_from(src: &TxProposalServiceModel) -> Result<Self, String> {
        let mcd_tx_proposal = mc_mobilecoind::payments::TxProposal::try_from(src)?;

        let tx_proposal = TxProposal::try_from(&mcd_tx_proposal)?;

        Ok(tx_proposal)
    }
}

impl TryFrom<&TxProposalServiceModel> for mc_mobilecoind::payments::TxProposal {
    type Error = String;

    #[allow(clippy::bind_instead_of_map)]
    fn try_from(
        src: &TxProposalServiceModel,
    ) -> Result<mc_mobilecoind::payments::TxProposal, String> {
        let unspent_txos: Vec<mc_mobilecoind::UnspentTxOut> = src
            .input_txos
            .iter()
            .map(|input_txo| mc_mobilecoind::UnspentTxOut {
                tx_out: input_txo.tx_out.clone(),
                subaddress_index: input_txo.subaddress_index,
                key_image: input_txo.key_image,
                value: input_txo.amount.value,
                attempted_spend_height: 0,
                attempted_spend_tombstone: 0,
                token_id: *input_txo.amount.token_id,
                memo_payload: MemoPayload::default().into(),
            })
            .collect();

        let mut outlay_list: Vec<mc_mobilecoind::payments::OutlayV2> = Vec::new();
        let mut outlay_map: HashMap<usize, usize> = HashMap::default();
        let mut confirmation_numbers: Vec<TxOutConfirmationNumber> = Vec::new();

        for (outlay_index, payload_txo) in src.payload_txos.iter().enumerate() {
            let tx_out_index = src
                .tx
                .prefix
                .outputs
                .iter()
                .enumerate()
                .position(|(_outlay_index, tx_out)| {
                    payload_txo.tx_out.public_key == tx_out.public_key
                })
                .ok_or("Could not find tx_out in tx")?;

            outlay_map.insert(outlay_index, tx_out_index);
            confirmation_numbers.push(payload_txo.confirmation_number.clone());
            outlay_list.push(mc_mobilecoind::payments::OutlayV2 {
                receiver: payload_txo.recipient_public_address.clone(),
                amount: payload_txo.amount,
                tx_private_key: None,
            });
        }

        let res = mc_mobilecoind::payments::TxProposal {
            utxos: unspent_txos,
            outlays: outlay_list,
            tx: src.tx.clone(),
            outlay_index_to_tx_out_index: outlay_map,
            outlay_confirmation_numbers: confirmation_numbers,
            scis: vec![],
        };

        Ok(res)
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
            fee: json_tx_proposal.fee.to_string().into(),
            outlay_index_to_tx_out_index: Secret::new(outlay_map),
            outlay_confirmation_numbers: Secret::new(
                json_tx_proposal.outlay_confirmation_numbers.clone(),
            ),
        })
    }
}

impl TryFrom<&TxProposal> for mc_mobilecoind::payments::TxProposal {
    type Error = String;

    #[allow(clippy::bind_instead_of_map)]
    fn try_from(src: &TxProposal) -> Result<mc_mobilecoind::payments::TxProposal, String> {
        // First, convert to the JsonTxProposal
        let json_tx_proposal = mc_mobilecoind_json::data_types::JsonTxProposal::try_from(src)
            .map_err(|err| format!("Failed to parse tx_proposal from json_rpc type {err:?}"))?;

        // Then convert to the proto tx proposal
        let proto_tx_proposal = mc_mobilecoind_api::TxProposal::try_from(&json_tx_proposal)
            .map_err(|err| format!("Failed to parse tx_proposal from json: {err:?}"))?;

        // Last, convert to the mobilecoind type
        let tx_proposal = mc_mobilecoind::payments::TxProposal::try_from(&proto_tx_proposal)
            .map_err(|err| format!("Failed to parse tx_proposal from proto: {err:?}"))?;
        Ok(tx_proposal)
    }
}

impl TryFrom<&TxProposal> for mc_mobilecoind_json::data_types::JsonTxProposal {
    type Error = String;

    #[allow(clippy::bind_instead_of_map)]
    fn try_from(
        src: &TxProposal,
    ) -> Result<mc_mobilecoind_json::data_types::JsonTxProposal, String> {
        let outlay_map: Vec<(usize, usize)> = src
            .outlay_index_to_tx_out_index
            .expose_secret()
            .iter()
            .map(|(key, val)| {
                key.parse::<usize>()
                    .and_then(|k| val.parse::<usize>().and_then(|v| Ok((k, v))))
                    .map_err(|err| format!("Failed to parse u64 from outlay_map: {err}"))
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
                .expose_secret()
                .parse::<u64>()
                .map_err(|err| format!("Failed to parse u64 from fee: {err}"))?,
            outlay_index_to_tx_out_index: outlay_map,
            outlay_confirmation_numbers: src.outlay_confirmation_numbers.expose_secret().clone(),
        })
    }
}
