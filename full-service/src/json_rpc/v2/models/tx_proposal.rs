// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the TxProposal object.

use super::amount::Amount as AmountJSON;
use crate::util::b58::{b58_encode_public_address, B58Error};
use redact::{expose_secret, Secret};
use serde_derive::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};

#[derive(Deserialize, Serialize, Default, Debug, PartialEq)]
pub struct UnsignedInputTxo {
    pub tx_out_proto: String,
    pub tx_out_public_key: String,
    pub amount: AmountJSON,
    pub subaddress_index: String,
}

#[derive(Clone, Deserialize, Serialize, Default, Debug, PartialEq)]
pub struct InputTxo {
    pub tx_out_proto: String,
    pub tx_out_public_key: String,
    pub amount: AmountJSON,
    pub subaddress_index: String,
    #[serde(serialize_with = "expose_secret")]
    pub key_image: Secret<String>,
}

#[derive(Clone, Deserialize, Serialize, Default, Debug, PartialEq)]
pub struct OutputTxo {
    pub tx_out_proto: String,
    pub tx_out_public_key: String,
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

impl From<&crate::service::models::tx_proposal::UnsignedInputTxo> for UnsignedInputTxo {
    fn from(src: &crate::service::models::tx_proposal::UnsignedInputTxo) -> Self {
        Self {
            tx_out_proto: hex::encode(mc_util_serial::encode(&src.tx_out)),
            tx_out_public_key: hex::encode(src.tx_out.public_key.as_bytes()),
            amount: AmountJSON::from(&src.amount),
            subaddress_index: src.subaddress_index.to_string(),
        }
    }
}

impl TryFrom<&UnsignedInputTxo> for crate::service::models::tx_proposal::UnsignedInputTxo {
    type Error = String;

    fn try_from(src: &UnsignedInputTxo) -> Result<Self, Self::Error> {
        let tx_out =
            mc_util_serial::decode(&hex::decode(&src.tx_out_proto).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;

        Ok(Self {
            tx_out,
            amount: (&src.amount).try_into()?,
            subaddress_index: src
                .subaddress_index
                .parse::<u64>()
                .map_err(|e| e.to_string())?,
        })
    }
}

impl TryFrom<&crate::service::models::tx_proposal::UnsignedTxProposal> for UnsignedTxProposal {
    type Error = String;

    fn try_from(
        src: &crate::service::models::tx_proposal::UnsignedTxProposal,
    ) -> Result<Self, Self::Error> {
        let unsigned_input_txos = src
            .unsigned_input_txos
            .iter()
            .map(|input_txo| input_txo.into())
            .collect();

        let payload_txos = src
            .payload_txos
            .iter()
            .map(|output_txo| {
                Ok(OutputTxo {
                    tx_out_proto: hex::encode(mc_util_serial::encode(&output_txo.tx_out)),
                    tx_out_public_key: hex::encode(output_txo.tx_out.public_key.as_bytes()),
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
                    tx_out_public_key: hex::encode(output_txo.tx_out.public_key.as_bytes()),
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
        let unsigned_tx_proto_bytes = mc_util_serial::encode(&unsigned_tx_external);
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
                tx_out_public_key: hex::encode(input_txo.tx_out.public_key.as_bytes()),
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
                    tx_out_public_key: hex::encode(output_txo.tx_out.public_key.as_bytes()),
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
                    tx_out_public_key: hex::encode(output_txo.tx_out.public_key.as_bytes()),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::create_test_txo_for_recipient;
    use mc_account_keys::AccountKey;
    use mc_transaction_core::{tokens::Mob, Amount, Token};
    use rand::{rngs::StdRng, SeedableRng};

    #[test]
    fn test_unsigned_input_txo_conversion() {
        let mut rng: StdRng = SeedableRng::from_seed([1u8; 32]);

        let recipient_account_key = AccountKey::random(&mut rng);
        let amount = Amount::new(1000, Mob::ID);

        let service_model = crate::service::models::tx_proposal::UnsignedInputTxo {
            tx_out: create_test_txo_for_recipient(&recipient_account_key, 12, amount, &mut rng).0,
            subaddress_index: 12,
            amount,
        };

        let json_rpc_model = UnsignedInputTxo::from(&service_model);
        let service_model_recovered =
            crate::service::models::tx_proposal::UnsignedInputTxo::try_from(&json_rpc_model)
                .unwrap();

        assert_eq!(service_model, service_model_recovered);
    }
}
