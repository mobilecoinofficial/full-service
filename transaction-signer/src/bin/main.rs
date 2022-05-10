use std::{convert::TryFrom, env, fs};

use mc_account_keys::AccountKey;
use mc_account_keys_slip10::Slip10Key;
use mc_common::HashMap;
use mc_crypto_keys::{RistrettoPrivate, RistrettoPublic};
use mc_fog_report_validation::FogResolver;
use mc_transaction_core::{
    get_tx_out_shared_secret,
    onetime_keys::{recover_onetime_private_key, recover_public_subaddress_spend_key},
    ring_signature::KeyImage,
    tx::TxOut,
    AmountError,
};

use mc_transaction_signer::UnsignedTx;

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
        create_account(output_file);

        return;
    }

    println!("Usage: {} <sign|check-txos|create-account>", args[0]);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Account {
    pub mnemonic_phrase: String,
    pub account_key: TSAccountKey,
}

/// The AccountKey contains a View keypair and a Spend keypair, used to
/// construct and receive transactions.
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct TSAccountKey {
    /// String representing the object's type. Objects of the same type share
    /// the same value.
    pub object: String,

    ///  Private key used for view-key matching, hex-encoded Ristretto bytes.
    pub view_private_key: String,

    /// Private key used for spending, hex-encoded Ristretto bytes.
    pub spend_private_key: String,

    /// Fog Report server url (if user has Fog service), empty string otherwise.
    pub fog_report_url: String,

    /// Fog Report Key (if user has Fog service), empty otherwise
    /// The key labelling the report to use, from among the several reports
    /// which might be served by the fog report server.
    pub fog_report_id: String,

    /// Fog Authority Subject Public Key Info (if user has Fog service),
    /// empty string otherwise.
    pub fog_authority_spki: String,
}

impl From<&AccountKey> for TSAccountKey {
    fn from(src: &AccountKey) -> TSAccountKey {
        TSAccountKey {
            object: "account_key".to_string(),
            view_private_key: ristretto_to_hex(src.view_private_key()),
            spend_private_key: ristretto_to_hex(src.spend_private_key()),
            fog_report_url: src.fog_report_url().unwrap_or("").to_string(),
            fog_report_id: src.fog_report_id().unwrap_or("").to_string(),
            fog_authority_spki: vec_to_hex(&src.fog_authority_spki().unwrap_or(&[]).to_vec()),
        }
    }
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

    let account_key = Slip10Key::from(mnemonic.clone())
        .try_into_account_key("", "", &base64::decode("").unwrap())
        .unwrap();

    let ts_account_key = TSAccountKey::from(&account_key);

    let account = Account {
        mnemonic_phrase: mnemonic.phrase().to_string(),
        account_key: ts_account_key,
    };

    let account_serialized = serde_json::to_string(&account).unwrap();
    fs::write(output_file, account_serialized).unwrap();
}

fn sign_transaction(
    unsigned_tx_file: &str,
    tombstone_block_height: &u64,
    signed_tx_file: &str,
    account_key: &AccountKey,
    subaddress_spend_public_keys: &HashMap<RistrettoPublic, u64>,
) {
    let unsigned_tx_bytes_serialized = fs::read_to_string(unsigned_tx_file).unwrap();
    let (unsigned_tx, fog_resolver): (UnsignedTx, FogResolver) =
        serde_json::from_str(&unsigned_tx_bytes_serialized).unwrap();

    let signed_tx = unsigned_tx.sign(
        account_key,
        subaddress_spend_public_keys,
        *tombstone_block_height,
        fog_resolver,
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
) -> Vec<(Vec<u8>, KeyImage)> {
    let mut serialized_txos_and_key_images: Vec<(Vec<u8>, KeyImage)> = Vec::new();

    for tx_out in tx_outs.iter() {
        if tx_out_belongs_to_account(tx_out, account_key.view_private_key()) {
            if let Some(key_image) =
                get_key_image_for_tx_out(tx_out, account_key, subaddress_spend_public_keys)
            {
                serialized_txos_and_key_images.push((mc_util_serial::encode(tx_out), key_image));
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
