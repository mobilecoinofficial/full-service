use mc_account_keys::{AccountKey, PublicAddress};
use mc_common::HashMap;
use mc_crypto_keys::{RistrettoPrivate, RistrettoPublic};
use mc_crypto_ring_signature_signer::NoKeysRingSigner;

use mc_transaction_core::{
    get_tx_out_shared_secret,
    onetime_keys::recover_onetime_private_key,
    ring_signature::{KeyImage, Scalar},
    tokens::Mob,
    tx::{TxIn, TxOut, TxOutConfirmationNumber},
    Amount, BlockVersion, Token, TokenId,
};
use mc_transaction_std::{
    InputCredentials, RTHMemoBuilder, ReservedSubaddresses, SenderMemoCredential,
    TransactionBuilder, TxOutContext,
};
use rand::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

use crate::{
    error::WalletTransactionBuilderError,
    fog_resolver::FullServiceFogResolver,
    service::models::tx_proposal::{InputTxo, OutputTxo, TxProposal},
    util::b58::b58_decode_public_address,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UnsignedTx {
    /// The fully constructed input rings
    pub inputs_and_real_indices_and_subaddress_indices: Vec<(TxIn, u64, u64)>,

    /// Vector of (PublicAddressB58, Amount, TokenId) for the recipients of this
    /// transaction.
    pub outlays: Vec<(String, u64, u64)>,

    /// The fee to be paid
    pub fee: u64,

    /// The fee's token id
    pub fee_token_id: u64,

    /// The tombstone block index
    pub tombstone_block_index: u64,

    /// The block version
    pub block_version: BlockVersion,
}

impl UnsignedTx {
    pub fn sign(
        self,
        account_key: &AccountKey,
        fog_resolver: FullServiceFogResolver,
    ) -> Result<TxProposal, WalletTransactionBuilderError> {
        todo!();
        // let mut rng = rand::thread_rng();
        // // Create transaction builder.
        // let mut memo_builder = RTHMemoBuilder::default();
        // memo_builder.set_sender_credential(SenderMemoCredential::
        // from(account_key)); memo_builder.enable_destination_memo();
        // let fee = Amount::new(self.fee, TokenId::from(self.fee_token_id));
        // let mut transaction_builder =
        //     TransactionBuilder::new(self.block_version, fee, fog_resolver,
        // memo_builder)?;

        // transaction_builder.set_tombstone_block(self.tombstone_block_index);

        // let mut inputs: Vec<InputTxo> = Vec::new();

        // for (tx_in, real_index, subaddress_index) in
        //     self.inputs_and_real_indices_and_subaddress_indices
        // {
        //     let tx_out = &tx_in.ring[real_index as usize];
        //     let tx_public_key =
        // RistrettoPublic::try_from(&tx_out.public_key)?;

        //     let onetime_private_key = recover_onetime_private_key(
        //         &tx_public_key,
        //         account_key.view_private_key(),
        //         &account_key.subaddress_spend_private(subaddress_index),
        //     );

        //     let key_image = KeyImage::from(&onetime_private_key);

        //     let input_credentials = InputCredentials::new(
        //         tx_in.ring.clone(),
        //         tx_in.proofs.clone(),
        //         real_index as usize,
        //         onetime_private_key,
        //         *account_key.view_private_key(),
        //     )?;

        //     transaction_builder.add_input(input_credentials);

        //     let tx_out = tx_in.ring[real_index as usize];
        //     let (amount, _) = decode_amount(&tx_out,
        // account_key.view_private_key())?;

        //     let input = InputTxo {
        //         tx_out,
        //         key_image,
        //         value: amount.value,
        //         token_id: amount.token_id,
        //     };

        //     inputs.push(input);
        // }

        // // Add the inputs and sum their values
        // let total_input_value = inputs.iter().map(|utxo| utxo.value as
        // u128).sum::<u128>() as u64;

        // let mut outlays_decoded: Vec<Outlay> = Vec::new();

        // for (public_address_b58, value) in self.outlays {
        //     let receiver = b58_decode_public_address(&public_address_b58)?;
        //     outlays_decoded.push(Outlay {
        //         receiver,
        //         value,
        //         token_id: Mob::ID,
        //     });
        // }

        // let (total_payload_value, tx_out_to_outlay_index,
        // outlay_confirmation_numbers) =     add_payload_outputs(&
        // outlays_decoded, &mut transaction_builder, &mut rng)?;

        // let change_txo = add_change_output(
        //     account_key,
        //     total_input_value,
        //     total_payload_value,
        //     &mut transaction_builder,
        //     &mut rng,
        // )?;

        // let tx = transaction_builder.build(&NoKeysRingSigner {}, &mut rng)?;

        // Ok(TxProposal {
        //     tx,
        //     input_txos: inputs,
        //     payload_txos: todo!(),
        //     change_txos: todo!(),
        // })
    }
}

pub fn decode_amount(
    tx_out: &TxOut,
    view_private_key: &RistrettoPrivate,
) -> Result<(Amount, Scalar), WalletTransactionBuilderError> {
    let tx_public_key = RistrettoPublic::try_from(&tx_out.public_key)?;
    let shared_secret = get_tx_out_shared_secret(view_private_key, &tx_public_key);
    Ok(tx_out.masked_amount.get_value(&shared_secret)?)
}

#[allow(clippy::type_complexity)]
fn add_payload_outputs<RNG: CryptoRng + RngCore>(
    outlays: &[(PublicAddress, u64, u64)],
    transaction_builder: &mut TransactionBuilder<FullServiceFogResolver>,
    rng: &mut RNG,
) -> Result<Vec<OutputTxo>, WalletTransactionBuilderError> {
    // Add outputs to our destinations.
    let mut outputs = Vec::new();
    for (recipient, value, token_id) in outlays.into_iter() {
        let token_id = TokenId::from(*token_id);
        let tx_out_context =
            transaction_builder.add_output(Amount::new(*value, token_id), recipient, rng)?;

        outputs.push(OutputTxo {
            tx_out: tx_out_context.tx_out,
            recipient_public_address: recipient.clone(),
            value: *value,
            token_id,
            confirmation_number: tx_out_context.confirmation,
        });
    }
    Ok(outputs)
}

fn add_change_output<RNG: CryptoRng + RngCore>(
    account_key: &AccountKey,
    total_input_value: u64,
    total_output_value: u64,
    token_id: TokenId,
    transaction_builder: &mut TransactionBuilder<FullServiceFogResolver>,
    rng: &mut RNG,
) -> Result<OutputTxo, WalletTransactionBuilderError> {
    let change_value = total_input_value - total_output_value;

    let reserved_subaddresses = ReservedSubaddresses::from(account_key);
    let tx_out_context = transaction_builder.add_change_output(
        Amount::new(change_value, token_id),
        &reserved_subaddresses,
        rng,
    )?;

    Ok(OutputTxo {
        tx_out: tx_out_context.tx_out,
        recipient_public_address: reserved_subaddresses.change_subaddress,
        value: change_value,
        token_id,
        confirmation_number: tx_out_context.confirmation,
    })
}
