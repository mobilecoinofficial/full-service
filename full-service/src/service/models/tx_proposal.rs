use std::convert::{TryFrom, TryInto};

use mc_account_keys::{AccountKey, PublicAddress};
use mc_api::ConversionError;
use mc_crypto_keys::RistrettoPublic;
use mc_crypto_ring_signature_signer::LocalRingSigner;
use mc_transaction_core::{
    onetime_keys::recover_onetime_private_key,
    ring_signature::KeyImage,
    tokens::Mob,
    tx::{Tx, TxOut},
    Amount, FeeMap, Token,
};
use mc_transaction_extra::{TxOutConfirmationNumber, UnsignedTx};
use protobuf::Message;

use crate::{service::transaction::TransactionServiceError, util::b58::b58_decode_public_address};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputTxo {
    pub tx_out: TxOut,
    pub subaddress_index: u64,
    pub key_image: KeyImage,
    pub amount: Amount,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnsignedInputTxo {
    pub tx_out: TxOut,
    pub subaddress_index: u64,
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

#[derive(Clone, Debug)]
pub struct UnsignedTxProposal {
    pub unsigned_tx: UnsignedTx,
    pub unsigned_input_txos: Vec<UnsignedInputTxo>,
    pub payload_txos: Vec<OutputTxo>,
    pub change_txos: Vec<OutputTxo>,
}

impl UnsignedTxProposal {
    pub fn sign(
        self,
        account_key: &AccountKey,
        fee_map: Option<&FeeMap>,
    ) -> Result<TxProposal, TransactionServiceError> {
        let input_txos = self
            .unsigned_input_txos
            .iter()
            .map(|txo| {
                let tx_out_public_key = RistrettoPublic::try_from(&txo.tx_out.public_key)?;
                let onetime_private_key = recover_onetime_private_key(
                    &tx_out_public_key,
                    account_key.view_private_key(),
                    &account_key.subaddress_spend_private(txo.subaddress_index),
                );

                let key_image = KeyImage::from(&onetime_private_key);

                Ok(InputTxo {
                    tx_out: txo.tx_out.clone(),
                    subaddress_index: txo.subaddress_index,
                    key_image,
                    amount: txo.amount,
                })
            })
            .collect::<Result<Vec<InputTxo>, TransactionServiceError>>()?;

        let signer = LocalRingSigner::from(account_key);
        let mut rng = rand::thread_rng();
        let tx = self.unsigned_tx.sign(&signer, fee_map, &mut rng)?;

        Ok(TxProposal {
            tx,
            input_txos,
            payload_txos: self.payload_txos,
            change_txos: self.change_txos,
        })
    }
}

impl TryFrom<crate::json_rpc::v2::models::tx_proposal::UnsignedTxProposal> for UnsignedTxProposal {
    type Error = String;

    fn try_from(
        src: crate::json_rpc::v2::models::tx_proposal::UnsignedTxProposal,
    ) -> Result<Self, Self::Error> {
        let unsigned_input_txos = src
            .unsigned_input_txos
            .iter()
            .map(|input_txo| {
                Ok(UnsignedInputTxo {
                    tx_out: mc_util_serial::decode(
                        hex::decode(&input_txo.tx_out_proto)
                            .map_err(|e| e.to_string())?
                            .as_slice(),
                    )
                    .map_err(|e| e.to_string())?,
                    subaddress_index: input_txo
                        .subaddress_index
                        .parse::<u64>()
                        .map_err(|e| e.to_string())?,
                    amount: Amount::try_from(&input_txo.amount)?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        let mut payload_txos = Vec::new();

        for txo in src.payload_txos.iter() {
            let confirmation_number_hex =
                hex::decode(&txo.confirmation_number).map_err(|e| format!("{e}"))?;
            let confirmation_number_bytes: [u8; 32] =
                confirmation_number_hex.as_slice().try_into().map_err(|_| {
                    "confirmation number is not the right number of bytes (expecting 32)"
                })?;
            let confirmation_number = TxOutConfirmationNumber::from(confirmation_number_bytes);

            let txo_out_hex = hex::decode(&txo.tx_out_proto).map_err(|e| e.to_string())?;
            let tx_out =
                mc_util_serial::decode(txo_out_hex.as_slice()).map_err(|e| e.to_string())?;
            let recipient_public_address =
                b58_decode_public_address(&txo.recipient_public_address_b58)
                    .map_err(|e| e.to_string())?;

            let amount = Amount::try_from(&txo.amount)?;

            let output_txo = OutputTxo {
                tx_out,
                recipient_public_address,
                confirmation_number,
                amount,
            };

            payload_txos.push(output_txo);
        }

        let mut change_txos = Vec::new();

        for txo in src.change_txos.iter() {
            let confirmation_number_hex =
                hex::decode(&txo.confirmation_number).map_err(|e| format!("{e}"))?;
            let confirmation_number_bytes: [u8; 32] =
                confirmation_number_hex.as_slice().try_into().map_err(|_| {
                    "confirmation number is not the right number of bytes (expecting 32)"
                })?;
            let confirmation_number = TxOutConfirmationNumber::from(confirmation_number_bytes);

            let txo_out_hex = hex::decode(&txo.tx_out_proto).map_err(|e| e.to_string())?;
            let tx_out =
                mc_util_serial::decode(txo_out_hex.as_slice()).map_err(|e| e.to_string())?;
            let recipient_public_address =
                b58_decode_public_address(&txo.recipient_public_address_b58)
                    .map_err(|e| e.to_string())?;

            let amount = Amount::try_from(&txo.amount)?;

            let output_txo = OutputTxo {
                tx_out,
                recipient_public_address,
                confirmation_number,
                amount,
            };

            change_txos.push(output_txo);
        }

        let proto_bytes =
            hex::decode(&src.unsigned_tx_proto_bytes_hex).map_err(|e| e.to_string())?;
        let unsigned_tx_external: mc_api::external::UnsignedTx =
            Message::parse_from_bytes(proto_bytes.as_slice()).map_err(|e| e.to_string())?;
        let unsigned_tx = (&unsigned_tx_external)
            .try_into()
            .map_err(|e: ConversionError| e.to_string())?;

        Ok(Self {
            unsigned_tx,
            unsigned_input_txos,
            payload_txos,
            change_txos,
        })
    }
}

impl TryFrom<&crate::json_rpc::v1::models::tx_proposal::TxProposal> for TxProposal {
    type Error = String;

    fn try_from(
        src: &crate::json_rpc::v1::models::tx_proposal::TxProposal,
    ) -> Result<Self, Self::Error> {
        let mc_api_tx = mc_api::external::Tx::try_from(&src.tx)?;
        let tx = Tx::try_from(&mc_api_tx).map_err(|e| e.to_string())?;

        let input_txos = src
            .input_list
            .iter()
            .map(|unspent_txo| {
                let mc_api_tx_out = mc_api::external::TxOut::try_from(&unspent_txo.tx_out)?;
                let tx_out = TxOut::try_from(&mc_api_tx_out).map_err(|e| e.to_string())?;

                let key_image_bytes =
                    hex::decode(unspent_txo.key_image.clone()).map_err(|e| e.to_string())?;
                let key_image =
                    KeyImage::try_from(key_image_bytes.as_slice()).map_err(|e| e.to_string())?;

                Ok(InputTxo {
                    tx_out,
                    subaddress_index: unspent_txo
                        .subaddress_index
                        .parse::<u64>()
                        .map_err(|e| e.to_string())?,
                    key_image,
                    amount: Amount::new(unspent_txo.value, Mob::ID),
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        let mut payload_txos = Vec::new();

        for (outlay_index, tx_out_index) in src.outlay_index_to_tx_out_index.iter() {
            let outlay_index = outlay_index.parse::<usize>().map_err(|e| e.to_string())?;
            let outlay = &src.outlay_list[outlay_index];
            let tx_out_index = tx_out_index.parse::<usize>().map_err(|e| e.to_string())?;
            let tx_out = tx.prefix.outputs[tx_out_index].clone();
            let confirmation_number_bytes: &[u8; 32] = src.outlay_confirmation_numbers
                [outlay_index]
                .as_slice()
                .try_into()
                .map_err(|_| {
                    "confirmation number is not the right number of bytes (expecting 32)"
                        .to_string()
                })?;

            let confirmation_number = TxOutConfirmationNumber::from(confirmation_number_bytes);

            let mc_api_public_address =
                mc_api::external::PublicAddress::try_from(&outlay.receiver)?;
            let public_address =
                PublicAddress::try_from(&mc_api_public_address).map_err(|e| e.to_string())?;

            let payload_txo = OutputTxo {
                tx_out,
                recipient_public_address: public_address,
                confirmation_number,
                amount: Amount::new(outlay.value.0, Mob::ID),
            };

            payload_txos.push(payload_txo);
        }

        Ok(Self {
            tx,
            input_txos,
            payload_txos,
            change_txos: Vec::new(),
        })
    }
}

impl TryFrom<&crate::json_rpc::v2::models::tx_proposal::TxProposal> for TxProposal {
    type Error = String;

    fn try_from(
        src: &crate::json_rpc::v2::models::tx_proposal::TxProposal,
    ) -> Result<Self, Self::Error> {
        let tx_bytes = hex::decode(&src.tx_proto).map_err(|e| e.to_string())?;
        let tx = mc_util_serial::decode(tx_bytes.as_slice()).map_err(|e| e.to_string())?;
        let input_txos = src
            .input_txos
            .iter()
            .map(|input_txo| {
                let key_image_bytes =
                    hex::decode(&input_txo.key_image.expose_secret()).map_err(|e| e.to_string())?;
                Ok(InputTxo {
                    tx_out: mc_util_serial::decode(
                        hex::decode(&input_txo.tx_out_proto)
                            .map_err(|e| e.to_string())?
                            .as_slice(),
                    )
                    .map_err(|e| e.to_string())?,
                    subaddress_index: input_txo
                        .subaddress_index
                        .parse::<u64>()
                        .map_err(|e| e.to_string())?,
                    key_image: KeyImage::try_from(key_image_bytes.as_slice())
                        .map_err(|e| e.to_string())?,
                    amount: Amount::try_from(&input_txo.amount)?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        let mut payload_txos = Vec::new();

        for txo in src.payload_txos.iter() {
            let confirmation_number_hex =
                hex::decode(&txo.confirmation_number).map_err(|e| format!("{e}"))?;
            let confirmation_number_bytes: [u8; 32] =
                confirmation_number_hex.as_slice().try_into().map_err(|_| {
                    "confirmation number is not the right number of bytes (expecting 32)"
                })?;
            let confirmation_number = TxOutConfirmationNumber::from(confirmation_number_bytes);

            let txo_out_hex = hex::decode(&txo.tx_out_proto).map_err(|e| e.to_string())?;
            let tx_out =
                mc_util_serial::decode(txo_out_hex.as_slice()).map_err(|e| e.to_string())?;
            let recipient_public_address =
                b58_decode_public_address(&txo.recipient_public_address_b58)
                    .map_err(|e| e.to_string())?;

            let amount = Amount::try_from(&txo.amount)?;

            let output_txo = OutputTxo {
                tx_out,
                recipient_public_address,
                confirmation_number,
                amount,
            };

            payload_txos.push(output_txo);
        }

        let mut change_txos = Vec::new();

        for txo in src.change_txos.iter() {
            let confirmation_number_hex =
                hex::decode(&txo.confirmation_number).map_err(|e| format!("{e}"))?;
            let confirmation_number_bytes: [u8; 32] =
                confirmation_number_hex.as_slice().try_into().map_err(|_| {
                    "confirmation number is not the right number of bytes (expecting 32)"
                })?;
            let confirmation_number = TxOutConfirmationNumber::from(confirmation_number_bytes);

            let txo_out_hex = hex::decode(&txo.tx_out_proto).map_err(|e| e.to_string())?;
            let tx_out =
                mc_util_serial::decode(txo_out_hex.as_slice()).map_err(|e| e.to_string())?;
            let recipient_public_address =
                b58_decode_public_address(&txo.recipient_public_address_b58)
                    .map_err(|e| e.to_string())?;

            let amount = Amount::try_from(&txo.amount)?;

            let output_txo = OutputTxo {
                tx_out,
                recipient_public_address,
                confirmation_number,
                amount,
            };

            change_txos.push(output_txo);
        }

        Ok(Self {
            tx,
            input_txos,
            payload_txos,
            change_txos,
        })
    }
}
