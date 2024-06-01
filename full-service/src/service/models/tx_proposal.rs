use std::convert::{TryFrom, TryInto};

use mc_account_keys::{AccountKey, PublicAddress};
use mc_api::ConversionError;
use mc_common::logger::global_log;
use mc_crypto_keys::RistrettoPublic;
use mc_crypto_ring_signature_signer::LocalRingSigner;
use mc_transaction_core::{
    onetime_keys::recover_onetime_private_key,
    ring_signature::KeyImage,
    tokens::Mob,
    tx::{Tx, TxOut},
    Amount, Token,
};
use mc_transaction_extra::{TxOutConfirmationNumber, UnsignedTx};

use protobuf::Message;

use crate::{
    db::{account::AccountModel, models::Account},
    service::{transaction::TransactionServiceError},
    util::b58::b58_decode_public_address,
};

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
    pub shared_secret: Option<RistrettoPublic>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TxProposal {
    pub tx: Tx,
    pub input_txos: Vec<InputTxo>,
    pub payload_txos: Vec<OutputTxo>,
    pub change_txos: Vec<OutputTxo>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UnsignedTxProposal {
    pub unsigned_tx: UnsignedTx,
    pub unsigned_input_txos: Vec<UnsignedInputTxo>,
    pub payload_txos: Vec<OutputTxo>,
    pub change_txos: Vec<OutputTxo>,
}

impl UnsignedTxProposal {
    pub async fn sign(self, account: &Account) -> Result<TxProposal, TransactionServiceError> {
        global_log::debug!("signing tx proposal with local signer");
        self.sign_with_local_signer(&account.account_key()?)
    }

    pub fn sign_with_local_signer(
        self,
        account_key: &AccountKey,
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
        let tx = self.unsigned_tx.sign(&signer, None, &mut rng)?;

        Ok(TxProposal {
            tx,
            input_txos,
            payload_txos: self.payload_txos,
            change_txos: self.change_txos,
        })
    }
}

impl TryFrom<&crate::json_rpc::v2::models::tx_proposal::UnsignedTxProposal> for UnsignedTxProposal {
    type Error = String;

    fn try_from(
        src: &crate::json_rpc::v2::models::tx_proposal::UnsignedTxProposal,
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

            let shared_secret = match &txo.shared_secret {
                Some(shared_secret) => {
                    let shared_secret_bytes =
                        hex::decode(shared_secret).map_err(|e| e.to_string())?;
                    Some(
                        RistrettoPublic::try_from(shared_secret_bytes.as_slice()).map_err(|e| {
                            format!("error converting shared secret to RistrettoPublic: {e}")
                        })?,
                    )
                }
                None => None,
            };

            let output_txo = OutputTxo {
                tx_out,
                recipient_public_address,
                confirmation_number,
                amount,
                shared_secret,
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

            let shared_secret = match &txo.shared_secret {
                Some(shared_secret) => {
                    let shared_secret_bytes =
                        hex::decode(shared_secret).map_err(|e| e.to_string())?;
                    Some(
                        RistrettoPublic::try_from(shared_secret_bytes.as_slice()).map_err(|e| {
                            format!("error converting shared secret to RistrettoPublic: {e}")
                        })?,
                    )
                }
                None => None,
            };

            let output_txo = OutputTxo {
                tx_out,
                recipient_public_address,
                confirmation_number,
                amount,
                shared_secret,
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

        for (outlay_index, tx_out_index) in src.outlay_index_to_tx_out_index.expose_secret().iter()
        {
            let outlay_index = outlay_index.parse::<usize>().map_err(|e| e.to_string())?;
            let outlay = &src.outlay_list[outlay_index];
            let tx_out_index = tx_out_index.parse::<usize>().map_err(|e| e.to_string())?;
            let tx_out = tx.prefix.outputs[tx_out_index].clone();
            let confirmation_number_bytes: &[u8; 32] =
                src.outlay_confirmation_numbers.expose_secret()[outlay_index]
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
                shared_secret: None,
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
                    hex::decode(input_txo.key_image.expose_secret()).map_err(|e| e.to_string())?;
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

            let shared_secret = match &txo.shared_secret {
                Some(shared_secret) => {
                    let shared_secret_bytes =
                        hex::decode(shared_secret).map_err(|e| e.to_string())?;
                    Some(
                        RistrettoPublic::try_from(shared_secret_bytes.as_slice()).map_err(|e| {
                            format!("error converting shared secret to RistrettoPublic: {e}")
                        })?,
                    )
                }
                None => None,
            };

            let output_txo = OutputTxo {
                tx_out,
                recipient_public_address,
                confirmation_number,
                amount,
                shared_secret,
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

            let shared_secret = match &txo.shared_secret {
                Some(shared_secret) => {
                    let shared_secret_bytes =
                        hex::decode(shared_secret).map_err(|e| e.to_string())?;
                    Some(
                        RistrettoPublic::try_from(shared_secret_bytes.as_slice()).map_err(|e| {
                            format!("error converting shared secret to RistrettoPublic: {e}")
                        })?,
                    )
                }
                None => None,
            };

            let output_txo = OutputTxo {
                tx_out,
                recipient_public_address,
                confirmation_number,
                amount,
                shared_secret,
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

#[cfg(test)]
mod tests {
    use crate::{
        db::account::AccountID,
        json_rpc::v2::models::amount::Amount as AmountJSON,
        service::{
            account::AccountService,
            address::AddressService,
            transaction::{TransactionMemo, TransactionService},
        },
        test_utils::{
            add_block_to_ledger_db, get_test_ledger, manually_sync_account, setup_wallet_service,
            MOB,
        },
    };

    use mc_common::logger::{test_with_logger, Logger};
    use mc_rand::RngCore;
    use rand::{rngs::StdRng, SeedableRng};

    use super::*;

    #[test_with_logger]
    fn test_v2_tx_proposal_converts_correctly(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        let service = setup_wallet_service(ledger_db.clone(), logger.clone());

        // Create our main account for the wallet
        let alice = service
            .create_account(
                Some("Alice's Main Account".to_string()),
                "".to_string(),
                "".to_string(),
                false,
            )
            .unwrap();

        // Add a block with a transaction for Alice
        let alice_account_key: AccountKey = mc_util_serial::decode(&alice.account_key).unwrap();
        let alice_account_id = AccountID::from(&alice_account_key);
        let alice_public_address = alice_account_key.default_subaddress();
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![alice_public_address.clone()],
            100 * MOB,
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        manually_sync_account(
            &ledger_db,
            &service.wallet_db.as_ref().unwrap(),
            &alice_account_id,
            &logger,
        );

        // Add an account for Bob
        let bob = service
            .create_account(
                Some("Bob's Main Account".to_string()),
                "".to_string(),
                "".to_string(),
                false,
            )
            .unwrap();

        // Create an assigned subaddress for Bob to receive funds from Alice
        let bob_address_from_alice = service
            .assign_address_for_account(&AccountID(bob.id.clone()), Some("From Alice"))
            .unwrap();

        // Create an assigned subaddress for Alice to receive from Bob, which will be
        // used to authenticate the sender (Alice)
        let alice_address_from_bob = service
            .assign_address_for_account(&alice_account_id, Some("From Bob"))
            .unwrap();

        let unsigned_tx_proposal = service
            .build_transaction(
                &alice.id,
                &[(
                    bob_address_from_alice.public_address_b58,
                    AmountJSON::new(42 * MOB, Mob::ID),
                )],
                None,
                None,
                None,
                None,
                None,
                TransactionMemo::RTH {
                    subaddress_index: Some(alice_address_from_bob.subaddress_index as u64),
                },
                None,
                None,
            )
            .unwrap();

        let unsigned_tx_proposal_v2_json_model =
            crate::json_rpc::v2::models::tx_proposal::UnsignedTxProposal::try_from(
                &unsigned_tx_proposal,
            )
            .unwrap();

        let unsigned_tx_proposal_converted_from_v2_json_model =
            UnsignedTxProposal::try_from(&unsigned_tx_proposal_v2_json_model).unwrap();

        assert_eq!(
            unsigned_tx_proposal,
            unsigned_tx_proposal_converted_from_v2_json_model
        );
    }
}
