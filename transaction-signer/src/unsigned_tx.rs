use mc_account_keys::AccountKey;
use mc_common::HashMap;
use mc_crypto_keys::{CompressedRistrettoPublic, RistrettoPrivate, RistrettoPublic};
use mc_transaction_core::{
    onetime_keys::{
        create_shared_secret, recover_onetime_private_key, recover_public_subaddress_spend_key,
    },
    ring_signature::{KeyImage, Scalar, SignatureRctBulletproofs},
    tx::{Tx, TxIn, TxOut, TxPrefix},
    AmountError, CompressedCommitment,
};
use mc_transaction_std::InputCredentials;

use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Clone, Deserialize, Serialize)]
pub struct UnsignedTx {
    /// List of input rings for the transaction, where each ring contains a
    /// single real input that is associated with the corresponding KeyImage
    pub inputs_and_key_images: Vec<(TxIn, KeyImage)>,

    /// List of outputs and shared secrets for the transaction
    pub outputs_and_shared_secrets: Vec<(TxOut, RistrettoPublic)>,

    /// Fee paid to the foundation for this transaction
    pub fee: u64,
}

impl UnsignedTx {
    pub fn sign(
        mut self,
        account_key: &AccountKey,
        subaddress_spend_public_keys: &HashMap<RistrettoPublic, u64>,
        tombstone_block_height: &u64,
    ) -> Tx {
        let mut rng = rand::thread_rng();

        let mut input_credentials: Vec<InputCredentials> = self
            .inputs_and_key_images
            .iter()
            .map(|(tx_in, key_image)| {
                let (real_index, onetime_private_key) =
                    real_index_and_onetime_private_key_for_ring(
                        tx_in,
                        key_image,
                        account_key,
                        subaddress_spend_public_keys,
                    )
                    .unwrap();
                let input_credential = InputCredentials::new(
                    tx_in.ring.clone(),
                    tx_in.proofs.clone(),
                    real_index as usize,
                    onetime_private_key,
                    *account_key.view_private_key(),
                )
                .unwrap();
                input_credential
            })
            .collect();

        // Construct a list of sorted inputs.
        // Inputs are sorted by the first ring element's public key. Note that each ring
        // is also sorted.
        input_credentials.sort_by(|a, b| a.ring[0].public_key.cmp(&b.ring[0].public_key));

        let inputs: Vec<TxIn> = input_credentials
            .iter()
            .map(|input_credential| TxIn {
                ring: input_credential.ring.clone(),
                proofs: input_credential.membership_proofs.clone(),
            })
            .collect();

        // Sort outputs by public key.
        self.outputs_and_shared_secrets
            .sort_by(|(a, _), (b, _)| a.public_key.cmp(&b.public_key));

        let output_values_and_blindings: Vec<(u64, Scalar)> = self
            .outputs_and_shared_secrets
            .iter()
            .map(|(tx_out, shared_secret)| {
                let amount = &tx_out.amount;
                let (value, blinding) = amount
                    .get_value(shared_secret)
                    .expect("TransactionBuilder created an invalid Amount");
                (value, blinding)
            })
            .collect();

        let (outputs, _shared_serets): (Vec<TxOut>, Vec<_>) =
            self.outputs_and_shared_secrets.into_iter().unzip();

        let tx_prefix = TxPrefix::new(inputs, outputs, self.fee, *tombstone_block_height);

        let mut rings: Vec<Vec<(CompressedRistrettoPublic, CompressedCommitment)>> = Vec::new();
        for input in &tx_prefix.inputs {
            let ring: Vec<(CompressedRistrettoPublic, CompressedCommitment)> = input
                .ring
                .iter()
                .map(|tx_out| (tx_out.target_key, tx_out.amount.commitment))
                .collect();
            rings.push(ring);
        }

        let real_input_indices: Vec<usize> = input_credentials
            .iter()
            .map(|input_credential| input_credential.real_index)
            .collect();

        // One-time private key, amount value, and amount blinding for each real input.
        let mut input_secrets: Vec<(RistrettoPrivate, u64, Scalar)> = Vec::new();
        for input_credential in &input_credentials {
            let onetime_private_key = input_credential.onetime_private_key;
            let amount = &input_credential.ring[input_credential.real_index].amount;
            let shared_secret = create_shared_secret(
                &input_credential.real_output_public_key,
                &input_credential.view_private_key,
            );
            let (value, blinding) = amount.get_value(&shared_secret).unwrap();
            input_secrets.push((onetime_private_key, value, blinding));
        }

        let message = tx_prefix.hash().0;
        let signature = SignatureRctBulletproofs::sign(
            &message,
            &rings,
            &real_input_indices,
            &input_secrets,
            &output_values_and_blindings,
            self.fee,
            &mut rng,
        )
        .unwrap();

        Tx {
            prefix: tx_prefix,
            signature,
        }
    }
}

fn real_index_and_onetime_private_key_for_ring(
    tx_in: &TxIn,
    key_image: &KeyImage,
    account_key: &AccountKey,
    subaddress_spend_public_keys: &HashMap<RistrettoPublic, u64>,
) -> Result<(u64, RistrettoPrivate), mc_transaction_core::AmountError> {
    for index in 0..tx_in.ring.len() {
        let tx_out = &tx_in.ring[index];
        if let Some(tx_out_onetime_private_key) =
            onetime_private_key_for_tx_out(&tx_out, account_key, subaddress_spend_public_keys)
        {
            let tx_out_key_image = KeyImage::from(&tx_out_onetime_private_key);
            if tx_out_key_image == *key_image {
                return Ok((index as u64, tx_out_onetime_private_key));
            }
        }
    }

    Err(AmountError::InconsistentCommitment)
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
