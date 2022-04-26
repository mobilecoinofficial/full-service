use mc_transaction_std::{InputCredentials, NoMemoBuilder, TransactionBuilder};
use serde::{Deserialize, Serialize};

use std::{convert::TryFrom, env, fs};

use mc_account_keys::AccountKey;
use mc_account_keys_slip10::Slip10Key;
use mc_common::HashMap;
use mc_crypto_keys::{RistrettoPrivate, RistrettoPublic};
use mc_transaction_core::{
    get_tx_out_shared_secret,
    onetime_keys::{recover_onetime_private_key, recover_public_subaddress_spend_key},
    ring_signature::KeyImage,
    tx::{Tx, TxIn, TxOut},
    AmountError,
};

use bip39::{Language, Mnemonic};
use mc_util_serial;

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
        &self,
        account_key: &AccountKey,
        subaddress_spend_public_keys: &HashMap<RistrettoPublic, u64>,
        tombstone_block_height: &u64,
    ) -> Tx {
        let mut rng = rand::thread_rng();
        // Collect all required FogUris from public addresses, then pass to
        // resolver factory
        // let fog_resolver = {
        // let fog_uris = core::slice::from_ref(&change_address)
        //     .iter()
        //     .chain(self.outlays.iter().map(|(receiver, _amount)|
        // receiver))     .filter_map(|x|
        // extract_fog_uri(x).transpose())     .collect::
        // <Result<Vec<_>, _>>()?; (self.fog_resolver_factory)(&
        // fog_uris)     .map_err(WalletTransactionBuilderError:
        // :FogPubkeyResolver)?
        // };

        // Create transaction builder.
        // TODO: After servers that support memos are deployed, use
        // RTHMemoBuilder here
        let memo_builder = NoMemoBuilder::default();
        let mut transaction_builder = TransactionBuilder::new(fog_resolver, memo_builder);
        transaction_builder.set_fee(self.fee).unwrap();
        transaction_builder.set_tombstone_block(*tombstone_block_height);

        self.inputs_and_key_images.iter().map(|(tx_in, key_image)| {
            let (real_index, onetime_private_key) = real_index_and_onetime_private_key_for_ring(
                tx_in,
                key_image,
                account_key,
                subaddress_spend_public_keys,
            )
            .unwrap();
            let input_credentials = InputCredentials::new(
                tx_in.ring,
                tx_in.proofs,
                real_index as usize,
                onetime_private_key,
                *account_key.view_private_key(),
            )
            .unwrap();
            transaction_builder.add_input(input_credentials);
        });

        transaction_builder.set_outputs_and_shared_secrets(self.outputs_and_shared_secrets);
        transaction_builder.build(&mut rng).unwrap()
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
