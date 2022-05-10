use mc_account_keys::{AccountKey, PublicAddress};
use mc_common::HashMap;
use mc_crypto_keys::{RistrettoPrivate, RistrettoPublic};
use mc_fog_report_validation::FogResolver;
use mc_transaction_core::{
    get_tx_out_shared_secret,
    onetime_keys::{recover_onetime_private_key, recover_public_subaddress_spend_key},
    ring_signature::Scalar,
    tx::{Tx, TxIn, TxOut},
    AmountError,
};
use mc_transaction_std::{ChangeDestination, InputCredentials, NoMemoBuilder, TransactionBuilder};

use rand::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Clone, Deserialize, Serialize)]
pub struct UnsignedTx {
    /// The fully constructed input rings
    pub inputs_and_real_indices: Vec<(TxIn, u64)>,

    /// Vector of (PublicAddress, Amounts) for the recipients of this
    /// transaction.
    pub outlays: Vec<(PublicAddress, u64)>,

    /// The fee to be paid
    pub fee: u64,
}

impl UnsignedTx {
    pub fn sign(
        self,
        account_key: &AccountKey,
        subaddress_spend_public_keys: &HashMap<RistrettoPublic, u64>,
        tombstone_block: u64,
        fog_resolver: FogResolver,
    ) -> Tx {
        let mut rng = rand::thread_rng();
        let memo_builder = NoMemoBuilder::default();

        let mut transaction_builder = TransactionBuilder::new(fog_resolver, memo_builder);
        transaction_builder.set_fee(self.fee).unwrap();
        transaction_builder.set_tombstone_block(tombstone_block);

        // Add the inputs and sum their values
        let total_input_value =
            self.inputs_and_real_indices
                .iter()
                .fold(0, |acc, (tx_in, real_index)| {
                    let onetime_private_key = onetime_private_key_for_tx_out(
                        &tx_in.ring[*real_index as usize],
                        account_key,
                        subaddress_spend_public_keys,
                    )
                    .unwrap();

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
                    let (amount, _) =
                        decode_amount(tx_out, account_key.view_private_key()).unwrap();
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

        transaction_builder.build(&mut rng).unwrap()
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
    outlays: Vec<(PublicAddress, u64)>,
    transaction_builder: &mut TransactionBuilder<FogResolver>,
    rng: &mut RNG,
) -> u64 {
    // Add outputs to our destinations.
    let mut total_value = 0;
    let mut tx_out_to_outlay_index: HashMap<TxOut, usize> = HashMap::default();
    let mut outlay_confirmation_numbers = Vec::default();
    for (i, (recipient, out_value)) in outlays.iter().enumerate() {
        let (tx_out, confirmation_number) = transaction_builder
            .add_output(*out_value as u64, &recipient, rng)
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
    transaction_builder: &mut TransactionBuilder<FogResolver>,
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

fn onetime_private_key_for_tx_out(
    tx_out: &TxOut,
    account_key: &AccountKey,
    subaddress_spend_public_keys: &HashMap<RistrettoPublic, u64>,
) -> Option<RistrettoPrivate> {
    let tx_public_key = match RistrettoPublic::try_from(&tx_out.public_key) {
        Ok(k) => k,
        Err(_) => return None,
    };
    let tx_out_target_key = match RistrettoPublic::try_from(&tx_out.target_key) {
        Ok(k) => k,
        Err(_) => return None,
    };

    let tx_out_subaddress_spend_public_key: RistrettoPublic = recover_public_subaddress_spend_key(
        account_key.view_private_key(),
        &tx_out_target_key,
        &tx_public_key,
    );

    let subaddress_index = subaddress_spend_public_keys
        .get(&tx_out_subaddress_spend_public_key)
        .copied();

    if let Some(subaddress_i) = subaddress_index {
        let onetime_private_key = recover_onetime_private_key(
            &tx_public_key,
            account_key.view_private_key(),
            &account_key.subaddress_spend_private(subaddress_i),
        );
        Some(onetime_private_key)
    } else {
        None
    }
}
