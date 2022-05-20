use mc_account_keys::AccountKey;
use mc_common::HashMap;
use mc_crypto_keys::{RistrettoPrivate, RistrettoPublic};
use mc_mobilecoind::{
    payments::{Outlay, TxProposal},
    UnspentTxOut,
};
use mc_transaction_core::{
    get_tx_out_shared_secret,
    onetime_keys::recover_onetime_private_key,
    ring_signature::{KeyImage, Scalar},
    tx::{TxIn, TxOut, TxOutConfirmationNumber},
};
use mc_transaction_std::{ChangeDestination, InputCredentials, NoMemoBuilder, TransactionBuilder};
use rand::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

use crate::{
    error::WalletTransactionBuilderError, fog_resolver::FullServiceFogResolver,
    util::b58::b58_decode_public_address,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UnsignedTx {
    /// The fully constructed input rings
    pub inputs_and_real_indices_and_subaddress_indices: Vec<(TxIn, u64, u64)>,

    /// Vector of (PublicAddressB58, Amount) for the recipients of this
    /// transaction.
    pub outlays: Vec<(String, u64)>,

    /// The fee to be paid
    pub fee: u64,

    /// The tombstone block index
    pub tombstone_block_index: u64,
}

impl UnsignedTx {
    pub fn sign(
        self,
        account_key: &AccountKey,
        fog_resolver: FullServiceFogResolver,
    ) -> Result<TxProposal, WalletTransactionBuilderError> {
        let mut rng = rand::thread_rng();
        let memo_builder = NoMemoBuilder::default();

        let mut transaction_builder = TransactionBuilder::new(fog_resolver, memo_builder);
        transaction_builder.set_fee(self.fee)?;
        transaction_builder.set_tombstone_block(self.tombstone_block_index);

        let mut selected_utxos: Vec<UnspentTxOut> = Vec::new();

        for (tx_in, real_index, subaddress_index) in
            self.inputs_and_real_indices_and_subaddress_indices
        {
            let tx_out = &tx_in.ring[real_index as usize];
            let tx_public_key = RistrettoPublic::try_from(&tx_out.public_key)?;

            let onetime_private_key = recover_onetime_private_key(
                &tx_public_key,
                account_key.view_private_key(),
                &account_key.subaddress_spend_private(subaddress_index),
            );

            let key_image = KeyImage::from(&onetime_private_key);

            let input_credentials = InputCredentials::new(
                tx_in.ring.clone(),
                tx_in.proofs.clone(),
                real_index as usize,
                onetime_private_key,
                *account_key.view_private_key(),
            )?;

            transaction_builder.add_input(input_credentials);

            let tx_out = &tx_in.ring[real_index as usize];
            let (value, _) = decode_amount(tx_out, account_key.view_private_key())?;

            let utxo = UnspentTxOut {
                tx_out: tx_out.clone(),
                subaddress_index: subaddress_index,
                key_image,
                value,
                attempted_spend_height: 0,
                attempted_spend_tombstone: 0,
            };

            selected_utxos.push(utxo);
        }

        // Add the inputs and sum their values
        let total_input_value = selected_utxos
            .iter()
            .map(|utxo| utxo.value as u128)
            .sum::<u128>() as u64;

        let mut outlays_decoded: Vec<Outlay> = Vec::new();

        for (public_address_b58, value) in self.outlays {
            let receiver = b58_decode_public_address(&public_address_b58)?;
            outlays_decoded.push(Outlay { receiver, value });
        }

        let (total_payload_value, tx_out_to_outlay_index, outlay_confirmation_numbers) =
            add_payload_outputs(&outlays_decoded, &mut transaction_builder, &mut rng)?;

        add_change_output(
            account_key,
            total_input_value,
            total_payload_value,
            &mut transaction_builder,
            &mut rng,
        )?;

        let tx = transaction_builder.build(&mut rng)?;

        let outlay_index_to_tx_out_index: HashMap<usize, usize> = tx
            .prefix
            .outputs
            .iter()
            .enumerate()
            .filter_map(|(tx_out_index, tx_out)| {
                tx_out_to_outlay_index
                    .get(tx_out)
                    .map(|outlay_index| (*outlay_index, tx_out_index))
            })
            .collect();

        Ok(TxProposal {
            utxos: selected_utxos,
            outlays: outlays_decoded.to_vec(),
            tx,
            outlay_index_to_tx_out_index,
            outlay_confirmation_numbers,
        })
    }
}

pub fn decode_amount(
    tx_out: &TxOut,
    view_private_key: &RistrettoPrivate,
) -> Result<(u64, Scalar), WalletTransactionBuilderError> {
    let tx_public_key = RistrettoPublic::try_from(&tx_out.public_key)?;
    let shared_secret = get_tx_out_shared_secret(view_private_key, &tx_public_key);
    Ok(tx_out.amount.get_value(&shared_secret)?)
}

fn add_payload_outputs<RNG: CryptoRng + RngCore>(
    outlays: &[Outlay],
    transaction_builder: &mut TransactionBuilder<FullServiceFogResolver>,
    rng: &mut RNG,
) -> Result<(u64, HashMap<TxOut, usize>, Vec<TxOutConfirmationNumber>), WalletTransactionBuilderError>
{
    // Add outputs to our destinations.
    let mut total_value = 0;
    let mut tx_out_to_outlay_index: HashMap<TxOut, usize> = HashMap::default();
    let mut outlay_confirmation_numbers = Vec::default();
    for (i, outlay) in outlays.iter().enumerate() {
        let (tx_out, confirmation_number) =
            transaction_builder.add_output(outlay.value, &outlay.receiver, rng)?;

        tx_out_to_outlay_index.insert(tx_out, i);
        outlay_confirmation_numbers.push(confirmation_number);

        total_value += outlay.value;
    }
    Ok((
        total_value,
        tx_out_to_outlay_index,
        outlay_confirmation_numbers,
    ))
}

fn add_change_output<RNG: CryptoRng + RngCore>(
    account_key: &AccountKey,
    total_input_value: u64,
    total_payload_value: u64,
    transaction_builder: &mut TransactionBuilder<FullServiceFogResolver>,
    rng: &mut RNG,
) -> Result<(), WalletTransactionBuilderError> {
    let change_value = total_input_value - total_payload_value - transaction_builder.get_fee();

    if change_value > 0 {
        let change_destination = ChangeDestination::from(account_key);
        transaction_builder.add_change_output(change_value, &change_destination, rng)?;
    }

    Ok(())
}
