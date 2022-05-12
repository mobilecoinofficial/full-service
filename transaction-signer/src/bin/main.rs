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

use mc_transaction_signer::{b58_encode_public_address, FullServiceFogResolver, UnsignedTx};

use bip39::{Language, Mnemonic, MnemonicType};
use serde::{Deserialize, Serialize};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} <sign|check-txos|create-account>", args[0]);
        return;
    }

    let operation = &args[1];

    if operation == "sign" {
        let unsigned_tx_file = &args[2];
        let tombstone_block_height = &args[3].parse::<u64>().unwrap();
        let signed_tx_file = &args[4];
        let account_key_config_file = &args[5];
        let num_subaddresses_to_check = &args[6].parse::<u64>().unwrap();

        let mnemonic_phrase = fs::read_to_string(account_key_config_file).unwrap();
        let account_key = account_key_from_mnemonic_phrase(&mnemonic_phrase);
        let subaddress_spend_public_keys =
            generate_subaddress_spend_public_keys(&account_key, *num_subaddresses_to_check);

        sign_transaction(
            unsigned_tx_file,
            tombstone_block_height,
            signed_tx_file,
            &account_key,
            &subaddress_spend_public_keys,
        );

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

    if operation == "create-account" {
        let output_file = &args[2];
        // TODO: Implement names for accounts
        // let name = &args[3];

        create_account(output_file);

        return;
    }

    if operation == "export-view-only-package" {
        let account_mnemonic_file = &args[2];
        let output_file = &args[3];

        export_view_only_package(account_mnemonic_file, output_file);
        return;
    }

    println!("Usage: {} <sign|check-txos|create-account>", args[0]);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ViewOnlyAccountPackage {
    view_private_key: String,
    main_subaddress_b58: String,
}

pub fn ristretto_to_vec(key: &RistrettoPrivate) -> Vec<u8> {
    mc_util_serial::encode(key)
}

pub fn vec_to_hex(key: &[u8]) -> String {
    hex::encode(key)
}

pub fn ristretto_to_hex(key: &RistrettoPrivate) -> String {
    vec_to_hex(&ristretto_to_vec(key))
}

fn create_account(output_file: &str) {
    let mnemonic = Mnemonic::new(MnemonicType::Words24, Language::English);

    fs::write(output_file, mnemonic.phrase().to_string()).unwrap();
}

fn export_view_only_package(account_mnemonic_file: &str, output_file: &str) {
    let mnemonic_phrase = fs::read_to_string(account_mnemonic_file).unwrap();
    let account_key = account_key_from_mnemonic_phrase(&mnemonic_phrase);

    let view_private_key = account_key.view_private_key();
    let main_subaddress = account_key.default_subaddress();
    let main_subaddress_b58 = b58_encode_public_address(&main_subaddress);

    let view_only_account_package = ViewOnlyAccountPackage {
        view_private_key: ristretto_to_hex(view_private_key),
        main_subaddress_b58,
    };

    let json = serde_json::to_string_pretty(&view_only_account_package).unwrap();

    fs::write(output_file, json).unwrap();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsignedTxAndFogResolver {
    pub unsigned_tx: UnsignedTx,
    pub fog_resolver: FullServiceFogResolver,
}

fn sign_transaction(
    unsigned_tx_file: &str,
    tombstone_block_height: &u64,
    signed_tx_file: &str,
    account_key: &AccountKey,
    subaddress_spend_public_keys: &HashMap<RistrettoPublic, u64>,
) {
    let unsigned_tx_bytes_serialized = fs::read_to_string(unsigned_tx_file).unwrap();
    let unsigned_tx_bundle: UnsignedTxAndFogResolver =
        serde_json::from_str(&unsigned_tx_bytes_serialized).unwrap();

    let signed_tx = unsigned_tx_bundle.unsigned_tx.sign(
        account_key,
        subaddress_spend_public_keys,
        *tombstone_block_height,
        unsigned_tx_bundle.fog_resolver,
    );

    let signed_tx_serialized = mc_util_serial::encode(&signed_tx);

    fs::write(signed_tx_file, signed_tx_serialized).unwrap();
}

fn account_key_from_mnemonic_phrase(mnemonic_phrase: &str) -> AccountKey {
    let mnemonic = Mnemonic::from_phrase(mnemonic_phrase, Language::English).unwrap();
    Slip10Key::from(mnemonic)
        .try_into_account_key("", "", &base64::decode("").unwrap())
        .unwrap()
}

fn check_txos(
    input_txos_file: &str,
    output_key_images_file: &str,
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
        get_key_images_for_txos(&input_txos, account_key, subaddress_spend_public_keys);
    let serialized_txos_and_key_images_data =
        serde_json::to_string(&serialized_txos_and_key_images).unwrap();
    fs::write(output_key_images_file, serialized_txos_and_key_images_data).unwrap();
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
) -> Vec<(Vec<u8>, Vec<u8>)> {
    let mut serialized_txos_and_key_images: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();

    for tx_out in tx_outs.iter() {
        if tx_out_belongs_to_account(tx_out, account_key.view_private_key()) {
            if let Some(key_image) =
                get_key_image_for_tx_out(tx_out, account_key, subaddress_spend_public_keys)
            {
                serialized_txos_and_key_images.push((
                    mc_util_serial::encode(tx_out),
                    mc_util_serial::encode(&key_image),
                ));
            }
        }
    }

    serialized_txos_and_key_images
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
