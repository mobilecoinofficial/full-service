use mc_account_keys::AccountKey;
use mc_common::HashMap;
use mc_crypto_keys::{RistrettoPrivate, RistrettoPublic};
use mc_mobilecoind::payments::TxProposal;
use mc_transaction_core::{
    get_tx_out_shared_secret,
    onetime_keys::recover_onetime_private_key,
    ring_signature::Scalar,
    tx::{Tx, TxIn, TxOut},
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

    /// Vector of (PublicAddress, Amounts) for the recipients of this
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

        // Add the inputs and sum their values
        let total_input_value = self
            .inputs_and_real_indices_and_subaddress_indices
            .iter()
            .fold(0, |acc, (tx_in, real_index, subaddress_index)| {
                let tx_out = &tx_in.ring[*real_index as usize];
                let tx_public_key = RistrettoPublic::try_from(&tx_out.public_key).unwrap();

                let onetime_private_key = recover_onetime_private_key(
                    &tx_public_key,
                    account_key.view_private_key(),
                    &account_key.subaddress_spend_private(*subaddress_index),
                );

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
                let (amount, _) = decode_amount(tx_out, account_key.view_private_key()).unwrap();
                acc + amount
            });

        let total_payload_value =
            add_payload_outputs(self.outlays, &mut transaction_builder, &mut rng);

        add_change_output(
            account_key,
            total_input_value,
            total_payload_value,
            &mut transaction_builder,
            &mut rng,
        );

        let tx = transaction_builder.build(&mut rng).unwrap();

        let selected_utxos = self.inputs_and_real_indices_and_subaddress_indices
        .iter()
        .map(|(tx_in, real_index, _)| {
            let tx_out = &tx_in.ring[*real_index as usize];
            let decoded_tx_out = mc_util_serial::decode(&tx_out.txo).unwrap();
            let decoded_key_image =
                mc_util_serial::decode(&tx_out.key_image.clone().unwrap()).unwrap();

            UnspentTxOut {
                tx_out: decoded_tx_out,
                subaddress_index: utxo.subaddress_index.unwrap() as u64, /* verified not null
                                                                          * earlier */
                key_image: decoded_key_image,
                value: utxo.value as u64,
                attempted_spend_height: 0, // NOTE: these are null because not tracked here
                attempted_spend_tombstone: 0,
            }
        })
        .collect();

        let tx_proposal = TxProposal {
            utxos: todo!(),
            outlays: todo!(),
            tx,
            outlay_index_to_tx_out_index: todo!(),
            outlay_confirmation_numbers: todo!(),
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
    outlays: Vec<(String, u64)>,
    transaction_builder: &mut TransactionBuilder<FullServiceFogResolver>,
    rng: &mut RNG,
) -> u64 {
    // Add outputs to our destinations.
    let mut total_value = 0;
    let mut tx_out_to_outlay_index: HashMap<TxOut, usize> = HashMap::default();
    let mut outlay_confirmation_numbers = Vec::default();
    for (i, (recipient, out_value)) in outlays.iter().enumerate() {
        let recipient_public_address = b58_decode_public_address(recipient).unwrap();
        let (tx_out, confirmation_number) = transaction_builder
            .add_output(*out_value as u64, &recipient_public_address, rng)
            .unwrap();

        tx_out_to_outlay_index.insert(tx_out, i);
        outlay_confirmation_numbers.push(confirmation_number);

        total_value += *out_value;
    }
    total_value
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
