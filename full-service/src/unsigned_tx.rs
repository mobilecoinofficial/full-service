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
    AmountError,
};
use mc_transaction_std::{ChangeDestination, InputCredentials, NoMemoBuilder, TransactionBuilder};
use rand::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

use crate::{fog_resolver::FullServiceFogResolver, util::b58::b58_decode_public_address};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UnsignedTx {
    /// The fully constructed input rings
    pub inputs_and_real_indices_and_subaddress_indices: Vec<(TxIn, u64, u64)>,

    /// Vector of (PublicAddressB58, Amount) for the recipients of this
    /// transaction.
    pub outlays: Vec<(String, u64)>,

    /// The fee to be paid
    pub fee: u64,
}

impl UnsignedTx {
    pub fn sign(
        self,
        account_key: &AccountKey,
        tombstone_block: u64,
        fog_resolver: FullServiceFogResolver,
    ) -> TxProposal {
        let mut rng = rand::thread_rng();
        let memo_builder = NoMemoBuilder::default();

        let mut minimum_fog_tombstone_block = u64::MAX;

        for fog_pubkey in fog_resolver.0.values() {
            minimum_fog_tombstone_block =
                std::cmp::min(minimum_fog_tombstone_block, fog_pubkey.pubkey_expiry);
        }

        if tombstone_block > minimum_fog_tombstone_block {
            panic!("Tombstone block {} is too far in the future compared to the minimum fog pubkey expiry {}", tombstone_block, minimum_fog_tombstone_block);
        }

        let mut transaction_builder = TransactionBuilder::new(fog_resolver, memo_builder);
        transaction_builder.set_fee(self.fee).unwrap();
        transaction_builder.set_tombstone_block(tombstone_block);

        let selected_utxos: Vec<UnspentTxOut> = self
            .inputs_and_real_indices_and_subaddress_indices
            .iter()
            .map(|(tx_in, real_index, subaddress_index)| {
                let tx_out = &tx_in.ring[*real_index as usize];
                let tx_public_key = RistrettoPublic::try_from(&tx_out.public_key).unwrap();

                let onetime_private_key = recover_onetime_private_key(
                    &tx_public_key,
                    account_key.view_private_key(),
                    &account_key.subaddress_spend_private(*subaddress_index),
                );

                let key_image = KeyImage::from(&onetime_private_key);

                let input_credentials = InputCredentials::new(
                    tx_in.ring.clone(),
                    tx_in.proofs.clone(),
                    *real_index as usize,
                    onetime_private_key,
                    *account_key.view_private_key(),
                )
                .unwrap();

                transaction_builder.add_input(input_credentials);

                let tx_out = &tx_in.ring[*real_index as usize];
                let (value, _) = decode_amount(tx_out, account_key.view_private_key()).unwrap();

                UnspentTxOut {
                    tx_out: tx_out.clone(),
                    subaddress_index: *subaddress_index,
                    key_image,
                    value,
                    attempted_spend_height: 0,
                    attempted_spend_tombstone: 0,
                }
            })
            .collect();

        // Add the inputs and sum their values
        let total_input_value = selected_utxos
            .iter()
            .map(|utxo| utxo.value as u128)
            .sum::<u128>() as u64;

        let outlays_decoded: &Vec<Outlay> = &self
            .outlays
            .iter()
            .map(|(public_address_b58, value)| {
                let receiver = b58_decode_public_address(public_address_b58).unwrap();
                Outlay {
                    value: *value,
                    receiver,
                }
            })
            .collect();

        let (total_payload_value, tx_out_to_outlay_index, outlay_confirmation_numbers) =
            add_payload_outputs(outlays_decoded, &mut transaction_builder, &mut rng);

        add_change_output(
            account_key,
            total_input_value,
            total_payload_value,
            &mut transaction_builder,
            &mut rng,
        );

        let tx = transaction_builder.build(&mut rng).unwrap();

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

        TxProposal {
            utxos: selected_utxos,
            outlays: outlays_decoded.to_vec(),
            tx,
            outlay_index_to_tx_out_index,
            outlay_confirmation_numbers,
        }
    }
}

pub fn decode_amount(
    tx_out: &TxOut,
    view_private_key: &RistrettoPrivate,
) -> Result<(u64, Scalar), AmountError> {
    let tx_public_key = RistrettoPublic::try_from(&tx_out.public_key).unwrap();
    let shared_secret = get_tx_out_shared_secret(view_private_key, &tx_public_key);
    tx_out.amount.get_value(&shared_secret)
}

fn add_payload_outputs<RNG: CryptoRng + RngCore>(
    outlays: &Vec<Outlay>,
    transaction_builder: &mut TransactionBuilder<FullServiceFogResolver>,
    rng: &mut RNG,
) -> (u64, HashMap<TxOut, usize>, Vec<TxOutConfirmationNumber>) {
    // Add outputs to our destinations.
    let mut total_value = 0;
    let mut tx_out_to_outlay_index: HashMap<TxOut, usize> = HashMap::default();
    let mut outlay_confirmation_numbers = Vec::default();
    for (i, outlay) in outlays.iter().enumerate() {
        let (tx_out, confirmation_number) = transaction_builder
            .add_output(outlay.value, &outlay.receiver, rng)
            .unwrap();

        tx_out_to_outlay_index.insert(tx_out, i);
        outlay_confirmation_numbers.push(confirmation_number);

        total_value += outlay.value;
    }
    (
        total_value,
        tx_out_to_outlay_index,
        outlay_confirmation_numbers,
    )
}

fn add_change_output<RNG: CryptoRng + RngCore>(
    account_key: &AccountKey,
    total_input_value: u64,
    total_payload_value: u64,
    transaction_builder: &mut TransactionBuilder<FullServiceFogResolver>,
    rng: &mut RNG,
) {
    let change_value = total_input_value - total_payload_value - transaction_builder.get_fee();

    if change_value > 0 {
        let change_destination = ChangeDestination::from(account_key);
        transaction_builder
            .add_change_output(change_value, &change_destination, rng)
            .unwrap();
    }
}
