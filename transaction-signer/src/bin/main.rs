use bip39::{Language, Mnemonic, MnemonicType};
use mc_account_keys::AccountKey;
use mc_common::HashMap;
use mc_core::slip10::Slip10KeyGenerator;
use mc_crypto_keys::{RistrettoPrivate, RistrettoPublic};
use mc_full_service::{
    db::{account::AccountID, txo::TxoID},
    json_rpc::{
        json_rpc_request::JsonRPCRequest,
        v2::{
            api::request::JsonCommandRequest,
            models::{
                account_key::AccountKey as AccountKeyJSON,
                account_secrets::AccountSecrets,
                tx_proposal::{
                    TxProposal as TxProposalJSON, UnsignedTxProposal as UnsignedTxProposalJSON,
                },
            },
        },
    },
    service::models::tx_proposal::UnsignedTxProposal,
    util::encoding_helpers::{ristretto_public_to_hex, ristretto_to_hex},
};
use mc_transaction_core::{
    get_tx_out_shared_secret,
    onetime_keys::{recover_onetime_private_key, recover_public_subaddress_spend_key},
    ring_signature::KeyImage,
    tx::TxOut,
};
use std::{convert::TryFrom, fs};
use structopt::StructOpt;

#[derive(Clone, Debug, StructOpt)]
#[structopt(
    name = "transaction-signer",
    about = "MobileCoin offline transaction signer"
)]
enum Opts {
    Create {
        #[structopt(short, long)]
        name: Option<String>,
    },
    Import {
        #[structopt(short, long)]
        name: Option<String>,
        mnemonic: String,
    },
    ImportFromLegecy {
        #[structopt(short, long)]
        name: Option<String>,
        entropy: String,
    },
    r#Sync {
        secret_mnemonic: String,
        sync_request: String,
        #[structopt(short, long, default_value = "1000")]
        subaddresses: u64,
    },
    Sign {
        secret_mnemonic: String,
        request: String,
    },
    ViewOnlyImportPackage {
        secret_mnemonic: String,
    },
}

fn main() {
    let opts = Opts::from_args();

    match opts {
        Opts::Create { ref name } => {
            let name = name.clone().unwrap_or_else(|| "".into());
            create_account(&name, None);
        }
        Opts::Import { mnemonic, name } => {
            let name = name.unwrap_or_else(|| "".into());
            create_account(&name, Some(&mnemonic));
        }
        Opts::ImportFromLegacy { entropy, name } => {
            let name = name.unwrap_or_else(|| "".into());
            import_account_from_legacy(&name, &entropy);
        }
        Opts::ViewOnlyImportPackage {
            ref secret_mnemonic,
        } => {
            generate_view_only_import_package(secret_mnemonic);
        }
        Opts::Sync {
            ref secret_mnemonic,
            ref sync_request,
            subaddresses,
        } => {
            sync_txos(secret_mnemonic, sync_request, subaddresses);
        }
        Opts::Sign {
            ref secret_mnemonic,
            ref request,
        } => {
            sign_transaction(secret_mnemonic, request);
        }
    }
}

fn import_account_from_legacy(name: &str, entropy: &str) {
    println!("Creating account {name} from legacy entropy");

    // Get account key from entropy
    let mut entropy_bytes = [0u8; 32];
    hex::decode_to_slice(entropy, &mut entropy_bytes)?;
    let root_entropy = RootEntropy::from(&entropy_bytes);
    let root_id = RootIdentity {
        root_entropy: entropy.clone()
    };
    let account_key = AccountKey::from(&root_id);

    let account_id = AccountID::from(&account_key);
    let view_private_key_hex = ristretto_to_hex(account_key.view_private_key());
    let spend_public_key = RistrettoPublic::from(account_key.spend_private_key());
    let spend_public_key_hex = ristretto_public_to_hex(&spend_public_key);

    let json_command_request = JsonCommandRequest::import_view_only_account {
        view_private_key: view_private_key_hex,
        spend_public_key: spend_public_key_hex,
        name: None,
        first_block_index: None,
        next_subaddress_index: None,
    };

    // Write view private key and associated info to file.
    let filename = format!(
        "mobilecoin_view_account_import_package_{}.json",
        &account_id.to_string()[..6]
    );
    write_json_command_request_to_file(&json_command_request, &filename);
}

fn create_account(name: &str, mnemonic: Option<&str>) {
    println!("Creating account {name}");

    let mnemonic = match mnemonic {
        Some(mnemonic) => Mnemonic::from_phrase(mnemonic, Language::English).unwrap(),
        None => Mnemonic::new(MnemonicType::Words24, Language::English),
    };

    let account_key = mnemonic.clone().derive_slip10_key(0).into();
    let account_id = AccountID::from(&account_key);

    let secrets = AccountSecrets {
        account_id: account_id.to_string(),
        entropy: None,
        mnemonic: Some(mnemonic.phrase().to_string()),
        key_derivation_version: "2".to_string(),
        account_key: Some(AccountKeyJSON::from(&account_key)),
        name: name.to_string(),
        view_account_key: None,
    };

    // Write secret mnemonic to file.
    let filename = format!(
        "mobilecoin_secret_mnemonic_{}.json",
        &account_id.to_string()[..6]
    );
    let output_json = serde_json::to_string_pretty(&secrets).unwrap();
    fs::write(&filename, output_json + "\n").expect("could not write output file");
    println!("Wrote {filename}");

    generate_view_only_import_package(&filename);
}

fn generate_view_only_import_package(secret_mnemonic: &str) {
    // Load account key.
    let mnemonic_json =
        fs::read_to_string(secret_mnemonic).expect("Could not open secret mnemonic file.");
    let account_secrets: AccountSecrets = serde_json::from_str(&mnemonic_json).unwrap();
    let account_key = account_key_from_mnemonic_phrase(&account_secrets.mnemonic.unwrap());
    let account_id = AccountID::from(&account_key);

    let view_private_key_hex = ristretto_to_hex(account_key.view_private_key());
    let spend_public_key = RistrettoPublic::from(account_key.spend_private_key());
    let spend_public_key_hex = ristretto_public_to_hex(&spend_public_key);

    let json_command_request = JsonCommandRequest::import_view_only_account {
        view_private_key: view_private_key_hex,
        spend_public_key: spend_public_key_hex,
        name: None,
        first_block_index: None,
        next_subaddress_index: None,
    };

    // Write view private key and associated info to file.
    let filename = format!(
        "mobilecoin_view_account_import_package_{}.json",
        &account_id.to_string()[..6]
    );
    write_json_command_request_to_file(&json_command_request, &filename);
}

fn sync_txos(secret_mnemonic: &str, sync_request: &str, num_subaddresses: u64) {
    // Load account key.
    let mnemonic_json =
        fs::read_to_string(secret_mnemonic).expect("Could not open secret mnemonic file.");
    let account_secrets: AccountSecrets = serde_json::from_str(&mnemonic_json).unwrap();
    let account_key = account_key_from_mnemonic_phrase(&account_secrets.mnemonic.unwrap());

    // Load input txos.
    let sync_request_data =
        fs::read_to_string(sync_request).expect("Could not open sync request file.");
    let sync_request_json: serde_json::Value =
        serde_json::from_str(&sync_request_data).expect("Malformed sync request.");
    let account_id = sync_request_json
        .get("account_id")
        .unwrap()
        .as_str()
        .unwrap();
    assert_eq!(account_secrets.account_id, account_id);

    let incomplete_txos_encoded: Vec<String> = serde_json::from_value(
        sync_request_json
            .get("incomplete_txos_encoded")
            .expect("Could not find \"incomplete_txos_encoded\".")
            .clone(),
    )
    .expect("Malformed sync request.");
    let input_txos: Vec<TxOut> = incomplete_txos_encoded
        .iter()
        .map(|tx_out_serialized| {
            mc_util_serial::decode(&hex::decode(tx_out_serialized.as_bytes()).unwrap()).unwrap()
        })
        .collect();

    // Generate subaddresses and reconstruct key images.
    let subaddress_spend_public_keys =
        generate_subaddress_spend_public_keys(&account_key, num_subaddresses);
    let txos_and_key_images =
        get_key_images_for_txos(&input_txos, &account_key, &subaddress_spend_public_keys);

    let completed_txos: Vec<(String, String)> = txos_and_key_images
        .iter()
        .map(|(txo, key_image, _)| {
            (
                TxoID::from(txo).to_string(),
                hex::encode(mc_util_serial::encode(key_image)),
            )
        })
        .collect();

    let json_command_request = JsonCommandRequest::sync_view_only_account {
        account_id: account_id.to_string(),
        completed_txos,
        next_subaddress_index: "0".to_string(),
    };

    // Write result to file.
    let filename = format!("{}_completed.json", sync_request.trim_end_matches(".json"));
    write_json_command_request_to_file(&json_command_request, &filename);
}

fn sign_transaction(secret_mnemonic: &str, sign_request: &str) {
    // Load account key.
    let mnemonic_json =
        fs::read_to_string(secret_mnemonic).expect("Could not open secret mnemonic file.");
    let account_secrets: AccountSecrets = serde_json::from_str(&mnemonic_json).unwrap();
    let account_key = account_key_from_mnemonic_phrase(&account_secrets.mnemonic.unwrap());

    // let signer = LocalRingSigner::from(&account_key);
    // let mut rng = rand::thread_rng();

    // // Load input txos.
    let request_data =
        fs::read_to_string(sign_request).expect("Could not open generate signing request file.");
    let request_json: serde_json::Value = serde_json::from_str(&request_data).expect(
        "Malformed generate signing
    request.",
    );
    let account_id = request_json.get("account_id").unwrap().as_str().unwrap();
    assert_eq!(account_secrets.account_id, account_id);

    let unsigned_tx_proposal_json: UnsignedTxProposalJSON = serde_json::from_value(
        request_json
            .get("unsigned_tx_proposal")
            .expect("Could not find \"unsigned_tx_proposal\".")
            .clone(),
    )
    .unwrap();

    let unsigned_tx_proposal: UnsignedTxProposal = unsigned_tx_proposal_json.try_into().unwrap();

    let tx_proposal = unsigned_tx_proposal.sign(&account_key, None).unwrap();
    let tx_proposal_json = TxProposalJSON::try_from(&tx_proposal).unwrap();
    let json_command_request = JsonCommandRequest::submit_transaction {
        tx_proposal: tx_proposal_json,
        comment: None,
        account_id: Some(account_id.to_string()),
    };

    let filename = format!(
        "{}_completed.json",
        sign_request.trim_end_matches("_unsigned.json")
    );
    write_json_command_request_to_file(&json_command_request, &filename);
}

fn write_json_command_request_to_file(json_command_request: &JsonCommandRequest, filename: &str) {
    let src_json: serde_json::Value = serde_json::json!(json_command_request);
    let method = src_json.get("method").unwrap().as_str().unwrap();
    let params = src_json.get("params").unwrap();

    let json_rpc_request = JsonRPCRequest {
        method: method.to_string(),
        params: Some(params.clone()),
        jsonrpc: "2.0".to_string(),
        id: serde_json::Value::Number(serde_json::Number::from(1)),
    };

    let result_json = serde_json::to_string_pretty(&json_rpc_request).unwrap();
    fs::write(filename, result_json + "\n").expect("could not write output file");
    println!("Wrote {filename}");
}

fn get_key_images_for_txos(
    tx_outs: &[TxOut],
    account_key: &AccountKey,
    subaddress_spend_public_keys: &HashMap<RistrettoPublic, u64>,
) -> Vec<(TxOut, KeyImage, u64)> {
    tx_outs
        .iter()
        .filter_map(|txo| {
            if !tx_out_belongs_to_account(txo, account_key.view_private_key()) {
                return None;
            }
            get_key_image_for_tx_out(txo, account_key, subaddress_spend_public_keys)
                .map(|(key_image, subaddress_index)| (txo.clone(), key_image, subaddress_index))
        })
        .collect()
}

fn account_key_from_mnemonic_phrase(mnemonic_phrase: &str) -> AccountKey {
    let mnemonic = Mnemonic::from_phrase(mnemonic_phrase, Language::English).unwrap();
    mnemonic.derive_slip10_key(0).into()
}

fn get_key_image_for_tx_out(
    tx_out: &TxOut,
    account_key: &AccountKey,
    subaddress_spend_public_keys: &HashMap<RistrettoPublic, u64>,
) -> Option<(KeyImage, u64)> {
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
        Some((KeyImage::from(&onetime_private_key), subaddress_i))
    } else {
        None
    }
}

fn tx_out_belongs_to_account(tx_out: &TxOut, account_view_private_key: &RistrettoPrivate) -> bool {
    let tx_out_public_key = match RistrettoPublic::try_from(&tx_out.public_key) {
        Err(_) => return false,
        Ok(k) => k,
    };
    let shared_secret = get_tx_out_shared_secret(account_view_private_key, &tx_out_public_key);
    tx_out
        .get_masked_amount()
        .unwrap()
        .get_value(&shared_secret)
        .is_ok()
}

fn generate_subaddress_spend_public_keys(
    account_key: &AccountKey,
    number_to_generate: u64,
) -> HashMap<RistrettoPublic, u64> {
    let mut subaddress_spend_public_keys = HashMap::default();

    let mut subaddresses: Vec<u64> = (0..number_to_generate).collect();
    subaddresses.push(mc_account_keys::CHANGE_SUBADDRESS_INDEX);
    for i in subaddresses.into_iter() {
        let subaddress_spend_private_key = account_key.subaddress_spend_private(i);
        let subaddress_spend_public_key = RistrettoPublic::from(&subaddress_spend_private_key);
        subaddress_spend_public_keys.insert(subaddress_spend_public_key, i);
    }

    subaddress_spend_public_keys
}
