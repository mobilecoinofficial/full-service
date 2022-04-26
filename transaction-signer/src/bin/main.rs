use std::{convert::TryFrom, env, fs};

use mc_account_keys::AccountKey;
use mc_account_keys_slip10::Slip10Key;
use mc_common::HashMap;
use mc_crypto_keys::{RistrettoPrivate, RistrettoPublic};
use mc_transaction_core::{
    get_tx_out_shared_secret,
    onetime_keys::{recover_onetime_private_key, recover_public_subaddress_spend_key},
    ring_signature::KeyImage,
    tx::TxOut,
    AmountError,
};

use mc_transaction_signer::UnsignedTx;

use bip39::{Language, Mnemonic};
use mc_util_serial;
use serde::Deserialize;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 6 || args.len() != 7 {
        println!("Usage: {} <sign|check-txos>", args[0]);
        return;
    }

    let operation = &args[1];

    if operation == "sign" {
        let unsigned_tx_file = &args[2];
        let tombstone_block_height = &args[3];
        let signed_tx_file = &args[4];
        let account_key_config_file = &args[5];
        let num_subaddresses_to_check = &args[6].parse::<u64>().unwrap();

        return;
    }

    if operation == "check-txos" {
        let input_txos_file = &args[2];
        let output_key_images_file = &args[3];
        let account_key_config_file = &args[4];
        let num_subaddresses_to_check = &args[5].parse::<u64>().unwrap();

        let mnemonic_phrase = fs::read_to_string(account_key_config_file).unwrap();
        let account_key = account_key_from_mnemonic_phrase(&mnemonic_phrase);
        let subaddress_spend_public_keys =
            generate_subaddress_spend_public_keys(&account_key, *num_subaddresses_to_check);

        check_txos(
            input_txos_file,
            output_key_images_file,
            &account_key,
            &subaddress_spend_public_keys,
        );

        return;
    }

    println!("Usage: {} <sign|check-txos>", args[0]);
}

fn account_key_from_mnemonic_phrase(mnemonic_phrase: &String) -> AccountKey {
    let mnemonic = Mnemonic::from_phrase(&mnemonic_phrase, Language::English).unwrap();
    let account_key = Slip10Key::from(mnemonic.clone())
        .try_into_account_key(&"", &"", &base64::decode("").unwrap())
        .unwrap();
    account_key
}

fn check_txos(
    input_txos_file: &String,
    output_key_images_file: &String,
    account_key: &AccountKey,
    subaddress_spend_public_keys: &HashMap<RistrettoPublic, u64>,
) {
    let input_txos_json = fs::read_to_string(input_txos_file).unwrap();
    let input_txos_serialized: Vec<Vec<u8>> = serde_json::from_str(&input_txos_json).unwrap();
    let input_txos: Vec<TxOut> = input_txos_serialized
        .iter()
        .map(|tx_out_serialized| {
            let tx_out: TxOut = mc_util_serial::decode(tx_out_serialized).unwrap();
            tx_out
        })
        .collect();
    let serialized_txos_and_key_images =
        get_key_images_for_txos(&input_txos, &account_key, subaddress_spend_public_keys);
    let serialized_txos_and_key_images =
        serde_json::to_string(&serialized_txos_and_key_images).unwrap();
    fs::write(output_key_images_file, serialized_txos_and_key_images).unwrap();
}

fn generate_subaddress_spend_public_keys(
    account_key: &AccountKey,
    number_to_generate: u64,
) -> HashMap<RistrettoPublic, u64> {
    let mut subaddress_spend_public_keys = HashMap::default();

    for i in 0..number_to_generate {
        let subaddress_spend_private_key = account_key.subaddress_spend_private(i);
        let subaddress_spend_public_key = RistrettoPublic::from(&subaddress_spend_private_key);
        subaddress_spend_public_keys.insert(subaddress_spend_public_key, i);
    }

    subaddress_spend_public_keys
}

fn get_key_images_for_txos(
    tx_outs: &[TxOut],
    account_key: &AccountKey,
    subaddress_spend_public_keys: &HashMap<RistrettoPublic, u64>,
) -> Vec<(Vec<u8>, KeyImage)> {
    let mut serialized_txos_and_key_images: Vec<Vec<u8>, KeyImage> = Vec::new();

    for tx_out in tx_outs.into_iter() {
        if tx_out_belongs_to_account(tx_out, account_key.view_private_key()) {
            if let Some(key_image) =
                get_key_image_for_tx_out(tx_out, account_key, subaddress_spend_public_keys)
            {
                serialized_txos_and_key_images.push((mc_util_serial::encode(&tx_out), key_image));
            }
        }
    }

    key_images
}

fn get_key_image_for_tx_out(
    tx_out: &TxOut,
    account_key: &AccountKey,
    subaddress_spend_public_keys: &HashMap<RistrettoPublic, u64>,
) -> Option<KeyImage> {
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

    let key_image = if let Some(subaddress_i) = subaddress_index {
        let onetime_private_key = recover_onetime_private_key(
            &tx_public_key,
            account_key.view_private_key(),
            &account_key.subaddress_spend_private(subaddress_i),
        );
        Some(KeyImage::from(&onetime_private_key))
    } else {
        None
    };

    key_image
}

fn tx_out_belongs_to_account(tx_out: &TxOut, account_view_private_key: &RistrettoPrivate) -> bool {
    let tx_out_public_key = match RistrettoPublic::try_from(&tx_out.public_key) {
        Err(_) => return false,
        Ok(k) => k,
    };

    let shared_secret = get_tx_out_shared_secret(account_view_private_key, &tx_out_public_key);

    match tx_out.amount.get_value(&shared_secret) {
        Ok((_, _)) => true,
        Err(AmountError::InconsistentCommitment) => false,
    }
}
