// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the TxBlueprint object.

use crate::{
    json_rpc::v2::models::{amount::Amount as AmountJSON, tx_proposal::UnsignedInputTxo},
    service::models::transaction_memo::TransactionMemo,
    util::b58::{b58_decode_public_address, b58_encode_public_address},
};
use mc_crypto_keys::{CompressedRistrettoPublic, RistrettoPublic};
use mc_transaction_builder::{TxBlueprint, TxOutContext};
use mc_transaction_extra::TxOutConfirmationNumber;
use serde_derive::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};

#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub struct TxBlueprintProposalTxoContext {
    pub tx_out_public_key: String,
    pub confirmation_number: String,
    pub shared_secret: String,
    pub recipient_public_address_b58: String,
    pub amount: AmountJSON,
}

impl TryFrom<&crate::service::models::tx_blueprint_proposal::TxBlueprintProposalTxoContext>
    for TxBlueprintProposalTxoContext
{
    type Error = String;

    fn try_from(
        src: &crate::service::models::tx_blueprint_proposal::TxBlueprintProposalTxoContext,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            tx_out_public_key: hex::encode(src.tx_out_context.tx_out_public_key.as_bytes()),
            confirmation_number: hex::encode(src.tx_out_context.confirmation.as_ref()),
            shared_secret: hex::encode(src.tx_out_context.shared_secret.to_bytes()),
            recipient_public_address_b58: b58_encode_public_address(&src.recipient_public_address)
                .map_err(|e| e.to_string())?,
            amount: AmountJSON::from(&src.amount),
        })
    }
}

impl TryFrom<&TxBlueprintProposalTxoContext>
    for crate::service::models::tx_blueprint_proposal::TxBlueprintProposalTxoContext
{
    type Error = String;

    fn try_from(src: &TxBlueprintProposalTxoContext) -> Result<Self, Self::Error> {
        let confirmation_number_hex =
            hex::decode(&src.confirmation_number).map_err(|e| e.to_string())?;
        let confirmation_number_bytes: [u8; 32] = confirmation_number_hex
            .as_slice()
            .try_into()
            .map_err(|_| "confirmation number is not the right number of bytes (expecting 32)")?;
        let confirmation = TxOutConfirmationNumber::from(confirmation_number_bytes);

        Ok(Self {
            tx_out_context: TxOutContext {
                tx_out_public_key: CompressedRistrettoPublic::try_from(
                    hex::decode(&src.tx_out_public_key)
                        .map_err(|e| e.to_string())?
                        .as_slice(),
                )
                .map_err(|e| e.to_string())?,
                confirmation,
                shared_secret: RistrettoPublic::try_from(
                    hex::decode(&src.shared_secret)
                        .map_err(|e| e.to_string())?
                        .as_slice(),
                )
                .map_err(|e| e.to_string())?,
            },
            recipient_public_address: b58_decode_public_address(&src.recipient_public_address_b58)
                .map_err(|e| e.to_string())?,
            amount: (&src.amount).try_into()?,
        })
    }
}

#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub struct TxBlueprintProposal {
    pub tx_blueprint_proto_bytes_hex: String,
    pub account_id_hex: String,
    pub memo: TransactionMemo,
    pub unsigned_input_txos: Vec<UnsignedInputTxo>,
    pub payload_txo_contexts: Vec<TxBlueprintProposalTxoContext>,
    pub change_txo_contexts: Vec<TxBlueprintProposalTxoContext>,
}

impl TryFrom<&crate::service::models::tx_blueprint_proposal::TxBlueprintProposal>
    for TxBlueprintProposal
{
    type Error = String;

    fn try_from(
        src: &crate::service::models::tx_blueprint_proposal::TxBlueprintProposal,
    ) -> Result<Self, Self::Error> {
        let tx_blueprint_external: mc_api::external::TxBlueprint = (&src.tx_blueprint).into();
        let tx_blueprint_proto_bytes = mc_util_serial::encode(&tx_blueprint_external);
        let tx_blueprint_proto_bytes_hex = hex::encode(tx_blueprint_proto_bytes.as_slice());

        Ok(Self {
            tx_blueprint_proto_bytes_hex,
            account_id_hex: src.account_id_hex.clone(),
            memo: src.memo.clone(),
            unsigned_input_txos: src
                .unsigned_input_txos
                .iter()
                .map(|txo| txo.into())
                .collect(),
            payload_txo_contexts: src
                .payload_txo_contexts
                .iter()
                .map(TxBlueprintProposalTxoContext::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            change_txo_contexts: src
                .change_txo_contexts
                .iter()
                .map(TxBlueprintProposalTxoContext::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl TryFrom<&TxBlueprintProposal>
    for crate::service::models::tx_blueprint_proposal::TxBlueprintProposal
{
    type Error = String;

    fn try_from(src: &TxBlueprintProposal) -> Result<Self, Self::Error> {
        let tx_blueprint_proto_bytes =
            hex::decode(&src.tx_blueprint_proto_bytes_hex).map_err(|e| e.to_string())?;
        let tx_blueprint_external: mc_api::external::TxBlueprint =
            mc_util_serial::decode(tx_blueprint_proto_bytes.as_slice())
                .map_err(|e| e.to_string())?;
        let tx_blueprint =
            TxBlueprint::try_from(&tx_blueprint_external).map_err(|e| e.to_string())?;
        Ok(Self {
            tx_blueprint,
            account_id_hex: src.account_id_hex.clone(),
            memo: src.memo.clone(),
            unsigned_input_txos: src
                .unsigned_input_txos
                .iter()
                .map(|txo| txo.try_into())
                .collect::<Result<Vec<_>, _>>()?,
            payload_txo_contexts: src
                .payload_txo_contexts
                .iter()
                .map(|output| output.try_into())
                .collect::<Result<Vec<_>, _>>()?,
            change_txo_contexts: src
                .change_txo_contexts
                .iter()
                .map(|output| output.try_into())
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::create_test_txo_for_recipient;
    use mc_account_keys::{AccountKey, PublicAddress};
    use mc_blockchain_types::BlockVersion;
    use mc_crypto_ring_signature_signer::NoKeysRingSigner;
    use mc_fog_report_validation_test_utils::MockFogResolver;
    use mc_transaction_builder::{
        test_utils::get_input_credentials, EmptyMemoBuilder, ReservedSubaddresses,
        SignedContingentInputBuilder, TransactionBuilder,
    };
    use mc_transaction_core::{
        constants::MILLIMOB_TO_PICOMOB, tokens::Mob, Amount, Token, TokenId,
    };
    use mc_util_from_random::FromRandom;
    use rand::{rngs::StdRng, SeedableRng};

    fn random_tx_blueprint_proposal_txo_context(
        rng: &mut StdRng,
    ) -> crate::service::models::tx_blueprint_proposal::TxBlueprintProposalTxoContext {
        let recipient = PublicAddress::from_random(rng);
        let amount = Amount::new(1000, Mob::ID);
        let tx_out_public_key = CompressedRistrettoPublic::from_random(rng);
        let confirmation = TxOutConfirmationNumber::from([1u8; 32]);
        let shared_secret = RistrettoPublic::from_random(rng);

        crate::service::models::tx_blueprint_proposal::TxBlueprintProposalTxoContext {
            tx_out_context: TxOutContext {
                tx_out_public_key,
                confirmation,
                shared_secret,
            },
            recipient_public_address: recipient,
            amount,
        }
    }

    fn random_unsigned_input_txo(
        rng: &mut StdRng,
    ) -> crate::service::models::tx_proposal::UnsignedInputTxo {
        let recipient_account_key = AccountKey::random(rng);
        let amount = Amount::new(1000, Mob::ID);

        crate::service::models::tx_proposal::UnsignedInputTxo {
            tx_out: create_test_txo_for_recipient(&recipient_account_key, 12, amount, rng).0,
            subaddress_index: 12,
            amount,
        }
    }

    #[test]
    fn test_tx_blueprint_txo_context_conversion() {
        let mut rng: StdRng = SeedableRng::from_seed([4u8; 32]);

        let service_model = random_tx_blueprint_proposal_txo_context(&mut rng);
        let json_rpc_model = TxBlueprintProposalTxoContext::try_from(&service_model).unwrap();
        let service_model_recovered =
            crate::service::models::tx_blueprint_proposal::TxBlueprintProposalTxoContext::try_from(
                &json_rpc_model,
            )
            .unwrap();

        assert_eq!(service_model, service_model_recovered);
    }

    #[test]
    fn test_tx_blueprint_proposal_conversion() {
        let mut rng: StdRng = SeedableRng::from_seed([1u8; 32]);
        let block_version = BlockVersion::MAX;

        let alice = AccountKey::random(&mut rng);
        let bob = AccountKey::random(&mut rng);
        let charlie = AccountKey::random(&mut rng);

        let token2 = TokenId::from(2);
        let fpr = MockFogResolver::default();

        let input_credentials_sci = get_input_credentials(
            block_version,
            Amount::new(1000, token2),
            &charlie,
            &fpr,
            &mut rng,
        );
        let proofs = input_credentials_sci.membership_proofs.clone();
        let mut sci_builder = SignedContingentInputBuilder::new(
            block_version,
            input_credentials_sci.clone(),
            fpr.clone(),
            EmptyMemoBuilder,
        )
        .unwrap();
        sci_builder
            .add_required_output(
                Amount::new(1000 * MILLIMOB_TO_PICOMOB, Mob::ID),
                &charlie.default_subaddress(),
                &mut rng,
            )
            .unwrap();
        let mut sci = sci_builder.build(&NoKeysRingSigner {}, &mut rng).unwrap();
        sci.tx_in.proofs = proofs;

        let mut transaction_builder = TransactionBuilder::new(
            block_version,
            Amount::new(Mob::MINIMUM_FEE, Mob::ID),
            fpr.clone(),
        )
        .unwrap();
        transaction_builder.add_input(get_input_credentials(
            block_version,
            Amount::new(1475 * MILLIMOB_TO_PICOMOB, Mob::ID),
            &alice,
            &fpr,
            &mut rng,
        ));
        transaction_builder.add_presigned_input(sci).unwrap();
        transaction_builder
            .add_output(
                Amount::new(1000, token2),
                &bob.default_subaddress(),
                &mut rng,
            )
            .unwrap();
        transaction_builder
            .add_change_output(
                Amount::new(475 * MILLIMOB_TO_PICOMOB - Mob::MINIMUM_FEE, Mob::ID),
                &ReservedSubaddresses::from(&alice),
                &mut rng,
            )
            .unwrap();

        let tx_blueprint = transaction_builder.build_blueprint().unwrap();

        let service_model = crate::service::models::tx_blueprint_proposal::TxBlueprintProposal {
            tx_blueprint,
            account_id_hex: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                .to_string(),
            memo: TransactionMemo::RTH {
                subaddress_index: Some(5),
            },
            unsigned_input_txos: vec![
                random_unsigned_input_txo(&mut rng),
                random_unsigned_input_txo(&mut rng),
            ],
            payload_txo_contexts: vec![
                random_tx_blueprint_proposal_txo_context(&mut rng),
                random_tx_blueprint_proposal_txo_context(&mut rng),
                random_tx_blueprint_proposal_txo_context(&mut rng),
            ],
            change_txo_contexts: vec![
                random_tx_blueprint_proposal_txo_context(&mut rng),
                random_tx_blueprint_proposal_txo_context(&mut rng),
            ],
        };
        let json_rpc_model = TxBlueprintProposal::try_from(&service_model).unwrap();
        let service_model_recovered =
            crate::service::models::tx_blueprint_proposal::TxBlueprintProposal::try_from(
                &json_rpc_model,
            )
            .unwrap();

        assert_eq!(service_model, service_model_recovered);
    }
}
