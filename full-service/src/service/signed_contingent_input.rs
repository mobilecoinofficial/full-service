// Copyright (c) 2020-2025 MobileCoin Inc.

//! Service for managing signed contingent inputs.

use displaydoc::Display;
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_fog_report_validation::FogPubkeyResolver;
use mc_ledger_db::{Error as LedgerError, Ledger};
use mc_transaction_extra::SignedContingentInput;
use mc_transaction_types::Amount;

use crate::{
    json_rpc::v2::models::signed_contingent_input::ValidateProofOfReserveSciResult, WalletService,
};

/// Errors for the SignedContingentInputService.
#[derive(Display, Debug)]
#[allow(clippy::large_enum_variant, clippy::result_large_err)]
pub enum SignedContingentInputServiceError {
    /// Error decoding prost: {0}
    ProstDecode(mc_util_serial::DecodeError),

    /// Error decoding from hex: {0}
    HexDecode(hex::FromHexError),

    /// Error from ledger: {0}
    LedgerError(LedgerError),
}

impl From<mc_util_serial::DecodeError> for SignedContingentInputServiceError {
    fn from(src: mc_util_serial::DecodeError) -> Self {
        Self::ProstDecode(src)
    }
}

impl From<hex::FromHexError> for SignedContingentInputServiceError {
    fn from(src: hex::FromHexError) -> Self {
        Self::HexDecode(src)
    }
}

impl From<LedgerError> for SignedContingentInputServiceError {
    fn from(src: LedgerError) -> Self {
        Self::LedgerError(src)
    }
}

/// Trait defining the ways in which the wallet can interact with and manage
/// signed contingent inputs.
#[rustfmt::skip]
#[allow(clippy::result_large_err)]
pub trait SignedContingentInputService {
    /// Validate a proof of reserve signed contingent input.
    /// This ensures the SCI is valid (has a valid signature), has a ring size of 1, is unspendable
    /// so no-one can consume it, and contains a real TxOut that appears in the ledger.
    /// Note that it does NOT check if the TxOut key image appears in the ledger so it is possible
    /// the TxOut has already been spent!
    ///
    /// # Arguments
    ///
    ///| Name               | Purpose                                                          | Notes                                                                             |
    ///|--------------------|------------------------------------------------------------------|-----------------------------------------------------------------------------------|
    ///| `sci_proto`        | The signed contingent input profobuf (hex-encoded)               |                                                                                   |
    ///
    fn validate_proof_of_reserve_sci(
        &self,
        sci_proto: &str,
    ) -> Result<ValidateProofOfReserveSciResult, SignedContingentInputServiceError>;
}

impl<T, FPR> SignedContingentInputService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn validate_proof_of_reserve_sci(
        &self,
        sci_proto: &str,
    ) -> Result<ValidateProofOfReserveSciResult, SignedContingentInputServiceError> {
        let sci: SignedContingentInput = mc_util_serial::decode(&hex::decode(sci_proto)?)?;

        if let Err(err) = sci.validate() {
            return Ok(ValidateProofOfReserveSciResult::InvalidSci {
                error: err.to_string(),
            });
        }

        if sci.tx_in.ring.len() != 1 {
            return Ok(ValidateProofOfReserveSciResult::NotProofOfReserveSci {
                error: "Proof of reserve SCI must have exactly one ring member".to_string(),
            });
        }

        let ring_txo = &sci.tx_in.ring[0];
        let tx_out_public_key = hex::encode(ring_txo.public_key);
        let txo_index = match self
            .ledger_db
            .get_tx_out_index_by_public_key(&ring_txo.public_key)
        {
            Ok(txo_index) => txo_index,
            Err(LedgerError::NotFound) => {
                return Ok(ValidateProofOfReserveSciResult::TxOutNotFoundInLedger {
                    tx_out_public_key,
                });
            }
            Err(e) => return Err(SignedContingentInputServiceError::LedgerError(e)),
        };
        let ledger_txo = match self.ledger_db.get_tx_out_by_index(txo_index) {
            Ok(txo) => txo,
            Err(LedgerError::NotFound) => {
                return Ok(ValidateProofOfReserveSciResult::TxOutNotFoundInLedger {
                    tx_out_public_key,
                });
            }
            Err(e) => return Err(SignedContingentInputServiceError::LedgerError(e)),
        };
        if ring_txo != &ledger_txo {
            return Ok(ValidateProofOfReserveSciResult::TxOutMismatch { tx_out_public_key });
        }

        if sci.required_output_amounts.len() != 1 {
            return Ok(ValidateProofOfReserveSciResult::NotProofOfReserveSci {
                error: "Proof of reserve SCI must have exactly one required output".to_string(),
            });
        }

        let required_output_amount = &sci.required_output_amounts[0];
        if required_output_amount.token_id != sci.pseudo_output_amount.token_id {
            return Ok(ValidateProofOfReserveSciResult::NotProofOfReserveSci {
                error: "Proof of reserve SCI must have the same token id for the required output as for the pseudo-output"
                    .to_string(),
            });
        }
        if required_output_amount.value != u64::MAX {
            return Ok(ValidateProofOfReserveSciResult::NotProofOfReserveSci {
                error: "Proof of reserve SCI must have the maximum value for the required output"
                    .to_string(),
            });
        }

        let Some(input_rules) = sci.tx_in.input_rules.as_ref() else {
            return Ok(ValidateProofOfReserveSciResult::NotProofOfReserveSci {
                error: "Proof of reserve SCI must have input rules".to_string(),
            });
        };
        if input_rules.max_tombstone_block != 1 {
            return Ok(ValidateProofOfReserveSciResult::NotProofOfReserveSci {
                error: "Proof of reserve SCI must have a max tombstone block of 1".to_string(),
            });
        }

        let amount = Amount::from(&sci.pseudo_output_amount);
        Ok(ValidateProofOfReserveSciResult::Valid {
            tx_out_public_key,
            key_image: hex::encode(sci.key_image()),
            amount: (&amount).into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{append_test_block, get_test_ledger, setup_wallet_service};
    use mc_account_keys::{AccountKey, DEFAULT_SUBADDRESS_INDEX};
    use mc_blockchain_types::BlockContents;
    use mc_common::logger::{async_test_with_logger, Logger};
    use mc_crypto_keys::RistrettoPublic;
    use mc_crypto_ring_signature_signer::{NoKeysRingSigner, OneTimeKeyDeriveData};
    use mc_fog_report_validation_test_utils::MockFogResolver;
    use mc_transaction_builder::{
        test_utils::get_ring, EmptyMemoBuilder, InputCredentials, SignedContingentInputBuilder,
    };
    use mc_transaction_core::{
        membership_proofs::Range,
        onetime_keys::recover_onetime_private_key,
        ring_signature::KeyImage,
        tokens::Mob,
        tx::{TxOutMembershipElement, TxOutMembershipProof},
        Token,
    };
    use mc_transaction_extra::SignedContingentInputError;
    use mc_transaction_types::{constants::MILLIMOB_TO_PICOMOB, BlockVersion, TokenId};
    use mc_util_from_random::FromRandom;
    use rand::{rngs::StdRng, CryptoRng, RngCore, SeedableRng};
    use std::convert::TryFrom;

    fn get_input_credentials_with_custom_ring_size<
        RNG: CryptoRng + RngCore,
        FPR: FogPubkeyResolver,
    >(
        ring_size: usize,
        block_version: BlockVersion,
        amount: Amount,
        account: &AccountKey,
        fog_resolver: &FPR,
        rng: &mut RNG,
    ) -> InputCredentials {
        let (ring, real_index) =
            get_ring(block_version, amount, ring_size, account, fog_resolver, rng);
        let real_output = ring[real_index].clone();

        let onetime_private_key = recover_onetime_private_key(
            &RistrettoPublic::try_from(&real_output.public_key).unwrap(),
            account.view_private_key(),
            &account.subaddress_spend_private(DEFAULT_SUBADDRESS_INDEX),
        );
        let onetime_key_derive_data = OneTimeKeyDeriveData::OneTimeKey(onetime_private_key);

        let membership_proofs: Vec<TxOutMembershipProof> = ring
            .iter()
            .map(|_tx_out| {
                // TransactionBuilder does not validate membership proofs, but does require one
                // for each ring member.
                TxOutMembershipProof::new(
                    real_index as u64,
                    ring.len() as u64,
                    (0..32)
                        .map(|_| TxOutMembershipElement::new(Range::new(0, 1).unwrap(), [2u8; 32]))
                        .collect(),
                )
            })
            .collect();
        assert_eq!(membership_proofs.len(), ring_size);
        assert_eq!(membership_proofs[0].elements.len(), 32);

        InputCredentials::new(
            ring,
            membership_proofs,
            real_index,
            onetime_key_derive_data,
            *account.view_private_key(),
        )
        .unwrap()
    }

    fn get_proof_of_reserve_sci_builder(
        rng: &mut StdRng,
    ) -> SignedContingentInputBuilder<MockFogResolver> {
        let block_version = BlockVersion::MAX;
        let fog_resolver = MockFogResolver(Default::default());

        let sender = AccountKey::random(rng);

        let value = 1475 * MILLIMOB_TO_PICOMOB;
        let amount = Amount::new(value, Mob::ID);

        let input_credentials = get_input_credentials_with_custom_ring_size(
            1,
            block_version,
            amount,
            &sender,
            &fog_resolver,
            rng,
        );

        let mut builder = SignedContingentInputBuilder::new(
            block_version,
            input_credentials,
            fog_resolver,
            EmptyMemoBuilder,
        )
        .unwrap();

        let amount2 = Amount::new(u64::MAX, amount.token_id);
        builder
            .add_required_output(amount2, &sender.default_subaddress(), rng)
            .unwrap();

        builder.set_tombstone_block(1);

        builder
    }

    #[async_test_with_logger]
    async fn test_validate_proof_of_reserve_sci_valid(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let mut ledger_db = get_test_ledger(5, &[], 12, &mut rng);
        let service = setup_wallet_service(ledger_db.clone(), None, logger.clone());

        let sci = get_proof_of_reserve_sci_builder(&mut rng)
            .build(&NoKeysRingSigner {}, &mut rng)
            .unwrap();
        let sci_proto = hex::encode(mc_util_serial::encode(&sci));

        let block_contents = BlockContents {
            key_images: vec![KeyImage::from(rng.next_u64())],
            outputs: sci.tx_in.ring.clone(),
            validated_mint_config_txs: vec![],
            mint_txs: vec![],
        };
        append_test_block(&mut ledger_db, block_contents, &mut rng);

        assert_eq!(
            service.validate_proof_of_reserve_sci(&sci_proto).unwrap(),
            ValidateProofOfReserveSciResult::Valid {
                tx_out_public_key: hex::encode(sci.tx_in.ring[0].public_key),
                key_image: hex::encode(sci.key_image()),
                amount: (&Amount::new(1475 * MILLIMOB_TO_PICOMOB, Mob::ID)).into(),
            }
        );
    }

    #[async_test_with_logger]
    async fn test_validate_proof_of_reserve_sci_invalid_sci(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let mut ledger_db = get_test_ledger(5, &[], 12, &mut rng);
        let service = setup_wallet_service(ledger_db.clone(), None, logger.clone());

        let valid_sci = get_proof_of_reserve_sci_builder(&mut rng)
            .build(&NoKeysRingSigner {}, &mut rng)
            .unwrap();

        let block_contents = BlockContents {
            key_images: vec![KeyImage::from(rng.next_u64())],
            outputs: valid_sci.tx_in.ring.clone(),
            validated_mint_config_txs: vec![],
            mint_txs: vec![],
        };
        append_test_block(&mut ledger_db, block_contents, &mut rng);

        // Mess with the key image
        let mut sci1 = valid_sci.clone();
        sci1.mlsag.key_image = KeyImage::from(rng.next_u64());

        // Mess with the amount
        let mut sci2 = valid_sci.clone();
        sci2.pseudo_output_amount.value += 1;

        // Mess with the token id
        let mut sci3 = valid_sci.clone();
        sci3.pseudo_output_amount.token_id += 1;

        for sci in [sci1, sci2, sci3] {
            let sci_proto = hex::encode(mc_util_serial::encode(&sci));
            assert_eq!(
                service.validate_proof_of_reserve_sci(&sci_proto).unwrap(),
                ValidateProofOfReserveSciResult::InvalidSci {
                    error: SignedContingentInputError::RingSignature(
                        mc_crypto_ring_signature::Error::InvalidSignature
                    )
                    .to_string(),
                }
            );
        }
    }

    #[async_test_with_logger]
    async fn test_validate_proof_of_reserve_sci_not_proof_of_reserve(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let mut ledger_db = get_test_ledger(5, &[], 12, &mut rng);
        let service = setup_wallet_service(ledger_db.clone(), None, logger.clone());

        let block_version = BlockVersion::MAX;
        let fog_resolver = MockFogResolver(Default::default());

        let sender = AccountKey::random(&mut rng);

        let value = 1475 * MILLIMOB_TO_PICOMOB;
        let amount = Amount::new(value, Mob::ID);

        let input_credentials = get_input_credentials_with_custom_ring_size(
            1,
            block_version,
            amount,
            &sender,
            &fog_resolver,
            &mut rng,
        );

        // Tombstone block is not 1
        {
            let mut sci_builder = get_proof_of_reserve_sci_builder(&mut rng);
            sci_builder.set_tombstone_block(2);
            let sci = sci_builder.build(&NoKeysRingSigner {}, &mut rng).unwrap();
            let block_contents = BlockContents {
                key_images: vec![KeyImage::from(rng.next_u64())],
                outputs: sci.tx_in.ring.clone(),
                validated_mint_config_txs: vec![],
                mint_txs: vec![],
            };
            append_test_block(&mut ledger_db, block_contents, &mut rng);

            let sci_proto = hex::encode(mc_util_serial::encode(&sci));
            assert_eq!(
                service.validate_proof_of_reserve_sci(&sci_proto).unwrap(),
                ValidateProofOfReserveSciResult::NotProofOfReserveSci {
                    error: "Proof of reserve SCI must have a max tombstone block of 1".to_string(),
                }
            );
        }

        // Multiple required outputs
        {
            let mut sci_builder = get_proof_of_reserve_sci_builder(&mut rng);
            sci_builder
                .add_required_output(
                    Amount::new(1475 * MILLIMOB_TO_PICOMOB, Mob::ID),
                    &sender.default_subaddress(),
                    &mut rng,
                )
                .unwrap();
            let sci = sci_builder.build(&NoKeysRingSigner {}, &mut rng).unwrap();
            let block_contents = BlockContents {
                key_images: vec![KeyImage::from(rng.next_u64())],
                outputs: sci.tx_in.ring.clone(),
                validated_mint_config_txs: vec![],
                mint_txs: vec![],
            };
            append_test_block(&mut ledger_db, block_contents, &mut rng);

            let sci_proto = hex::encode(mc_util_serial::encode(&sci));
            assert_eq!(
                service.validate_proof_of_reserve_sci(&sci_proto).unwrap(),
                ValidateProofOfReserveSciResult::NotProofOfReserveSci {
                    error: "Proof of reserve SCI must have exactly one required output".to_string(),
                }
            );
        }

        // Required output amount is not the maximum value
        {
            let mut sci_builder = SignedContingentInputBuilder::new(
                block_version,
                input_credentials.clone(),
                fog_resolver.clone(),
                EmptyMemoBuilder,
            )
            .unwrap();

            let amount2 = Amount::new(u64::MAX - 1, amount.token_id);
            sci_builder
                .add_required_output(amount2, &sender.default_subaddress(), &mut rng)
                .unwrap();

            sci_builder.set_tombstone_block(1);

            let sci = sci_builder.build(&NoKeysRingSigner {}, &mut rng).unwrap();
            let block_contents = BlockContents {
                key_images: vec![KeyImage::from(rng.next_u64())],
                outputs: sci.tx_in.ring.clone(),
                validated_mint_config_txs: vec![],
                mint_txs: vec![],
            };
            append_test_block(&mut ledger_db, block_contents, &mut rng);

            let sci_proto = hex::encode(mc_util_serial::encode(&sci));
            assert_eq!(
                service.validate_proof_of_reserve_sci(&sci_proto).unwrap(),
                ValidateProofOfReserveSciResult::NotProofOfReserveSci {
                    error:
                        "Proof of reserve SCI must have the maximum value for the required output"
                            .to_string(),
                }
            );
        }

        // Required output token id is not the same as the input token id
        {
            let mut sci_builder = SignedContingentInputBuilder::new(
                block_version,
                input_credentials,
                fog_resolver.clone(),
                EmptyMemoBuilder,
            )
            .unwrap();

            let amount2 = Amount::new(u64::MAX, TokenId::from(123));
            sci_builder
                .add_required_output(amount2, &sender.default_subaddress(), &mut rng)
                .unwrap();

            sci_builder.set_tombstone_block(1);

            let sci = sci_builder.build(&NoKeysRingSigner {}, &mut rng).unwrap();

            let sci_proto = hex::encode(mc_util_serial::encode(&sci));
            assert_eq!(
                service.validate_proof_of_reserve_sci(&sci_proto).unwrap(),
                ValidateProofOfReserveSciResult::NotProofOfReserveSci {
                    error: "Proof of reserve SCI must have the same token id for the required output as for the pseudo-output"
                        .to_string(),
                }
            );
        }

        // Must have exactly one ring member
        {
            let input_credentials = get_input_credentials_with_custom_ring_size(
                2,
                block_version,
                amount,
                &sender,
                &fog_resolver,
                &mut rng,
            );

            let mut sci_builder = SignedContingentInputBuilder::new(
                block_version,
                input_credentials,
                fog_resolver,
                EmptyMemoBuilder,
            )
            .unwrap();

            let amount2 = Amount::new(u64::MAX, TokenId::from(123));
            sci_builder
                .add_required_output(amount2, &sender.default_subaddress(), &mut rng)
                .unwrap();

            sci_builder.set_tombstone_block(1);

            let sci = sci_builder.build(&NoKeysRingSigner {}, &mut rng).unwrap();

            let sci_proto = hex::encode(mc_util_serial::encode(&sci));
            assert_eq!(
                service.validate_proof_of_reserve_sci(&sci_proto).unwrap(),
                ValidateProofOfReserveSciResult::NotProofOfReserveSci {
                    error: "Proof of reserve SCI must have exactly one ring member".to_string(),
                }
            );
        }
    }

    #[async_test_with_logger]
    async fn test_validate_proof_of_reserve_sci_tx_not_in_ledger(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let ledger_db = get_test_ledger(5, &[], 12, &mut rng);
        let service = setup_wallet_service(ledger_db.clone(), None, logger.clone());

        let sci = get_proof_of_reserve_sci_builder(&mut rng)
            .build(&NoKeysRingSigner {}, &mut rng)
            .unwrap();
        let sci_proto = hex::encode(mc_util_serial::encode(&sci));

        // Initially the TxOut is not in the ledger.
        assert_eq!(
            service.validate_proof_of_reserve_sci(&sci_proto).unwrap(),
            ValidateProofOfReserveSciResult::TxOutNotFoundInLedger {
                tx_out_public_key: hex::encode(sci.tx_in.ring[0].public_key),
            }
        );
    }

    #[async_test_with_logger]
    async fn test_validate_proof_of_reserve_sci_txo_mismatch(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
        let mut ledger_db = get_test_ledger(5, &[], 12, &mut rng);
        let service = setup_wallet_service(ledger_db.clone(), None, logger.clone());

        let sci = get_proof_of_reserve_sci_builder(&mut rng)
            .build(&NoKeysRingSigner {}, &mut rng)
            .unwrap();
        let sci_proto = hex::encode(mc_util_serial::encode(&sci));

        let mut block_contents = BlockContents {
            key_images: vec![KeyImage::from(rng.next_u64())],
            outputs: sci.tx_in.ring.clone(),
            validated_mint_config_txs: vec![],
            mint_txs: vec![],
        };
        block_contents.outputs[0].target_key = (&RistrettoPublic::from_random(&mut rng)).into();
        append_test_block(&mut ledger_db, block_contents, &mut rng);

        assert_eq!(
            service.validate_proof_of_reserve_sci(&sci_proto).unwrap(),
            ValidateProofOfReserveSciResult::TxOutMismatch {
                tx_out_public_key: hex::encode(sci.tx_in.ring[0].public_key),
            }
        );
    }
}
