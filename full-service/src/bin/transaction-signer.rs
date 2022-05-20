use bip39::{Language, Mnemonic, MnemonicType};
use mc_account_keys::{AccountKey, CHANGE_SUBADDRESS_INDEX, DEFAULT_SUBADDRESS_INDEX};
use mc_account_keys_slip10::Slip10Key;
use mc_common::{HashMap, HashSet};
use mc_full_service::{
    db::{account::AccountID, txo::TxoID},
    fog_resolver::FullServiceFogResolver,
    json_rpc::{
        account_key::AccountKey as AccountKeyJSON,
        account_secrets::AccountSecrets,
        json_rpc_request::{JsonCommandRequest, JsonRPCRequest},
        tx_proposal::TxProposal,
        view_only_account::{ViewOnlyAccountJSON, ViewOnlyAccountSecretsJSON},
        view_only_subaddress::ViewOnlySubaddressJSON,
    },
    unsigned_tx::UnsignedTx,
    util::b58,
};
use std::{convert::TryFrom, fs};
use structopt::StructOpt;

use mc_crypto_keys::{RistrettoPrivate, RistrettoPublic};

use mc_transaction_core::{
    get_tx_out_shared_secret,
    onetime_keys::{recover_onetime_private_key, recover_public_subaddress_spend_key},
    ring_signature::KeyImage,
    tx::TxOut,
    AmountError,
};

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
    r#Sync {
        secret_mnemonic: String,
        sync_request: String,
        #[structopt(short, long, default_value = "1000")]
        subaddresses: u64,
    },
    Subaddresses {
        secret_mnemonic: String,
        request: String,
    },
    Sign {
        secret_mnemonic: String,
        request: String,
    },
}

fn main() {
    let opts = Opts::from_args();

    match opts {
        Opts::Create { ref name } => {
            let name = name.clone().unwrap_or_else(|| "".into());
            create_account(&name);
        }
        Opts::Sync {
            ref secret_mnemonic,
            ref sync_request,
            subaddresses,
        } => {
            sync_txos(secret_mnemonic, sync_request, subaddresses);
        }
        Opts::Subaddresses {
            ref secret_mnemonic,
            ref request,
        } => {
            generate_subaddresses(secret_mnemonic, request);
        }
        Opts::Sign {
            ref secret_mnemonic,
            ref request,
        } => {
            sign_transaction(secret_mnemonic, request);
        }
    }
}

fn create_account(name: &str) {
    println!("Creating account {}", name);

    // Generate new seed mnemonic.
    let mnemonic = Mnemonic::new(MnemonicType::Words24, Language::English);

    let fog_report_url = "".to_string();
    let fog_report_id = "".to_string();
    let fog_authority_spki = "".to_string();
    let account_key = Slip10Key::from(mnemonic.clone())
        .try_into_account_key(
            &fog_report_url,
            &fog_report_id,
            &base64::decode(fog_authority_spki).expect("Invalid Fog SPKI"),
        )
        .expect("could not generate account key");
    let account_id = AccountID::from(&account_key);

    let secrets = AccountSecrets {
        object: "account_secrets".to_string(),
        account_id: account_id.to_string(),
        entropy: None,
        mnemonic: Some(mnemonic.phrase().to_string()),
        key_derivation_version: "2".to_string(),
        account_key: AccountKeyJSON::from(&account_key),
        name: name.to_string(),
    };

    // Package view private key.
    let account_json = ViewOnlyAccountJSON {
        object: "view_only_account".to_string(),
        name: name.to_string(),
        account_id: account_id.to_string(),
        first_block_index: 0.to_string(),
        next_block_index: 0.to_string(),
        main_subaddress_index: DEFAULT_SUBADDRESS_INDEX.to_string(),
        change_subaddress_index: CHANGE_SUBADDRESS_INDEX.to_string(),
        next_subaddress_index: 2.to_string(),
    };

    let account_secrets_json = ViewOnlyAccountSecretsJSON {
        object: "view_only_account_secrets".to_string(),
        view_private_key: hex::encode(mc_util_serial::encode(account_key.view_private_key())),
        account_id: account_id.to_string(),
    };

    // Generate main and change subaddresses.
    let initial_subaddresses = vec![
        subaddress_json(&account_key, DEFAULT_SUBADDRESS_INDEX, "Main"),
        subaddress_json(&account_key, CHANGE_SUBADDRESS_INDEX, "Change"),
    ];

    // Write secret mnemonic to file.
    let filename = format!(
        "mobilecoin_secret_mnemonic_{}.json",
        &account_id.to_string()[..6]
    );
    let output_json = serde_json::to_string_pretty(&secrets).unwrap();
    fs::write(&filename, output_json + "\n").expect("could not write output file");
    println!("Wrote {}", filename);

    let json_command_request = JsonCommandRequest::import_view_only_account {
        account: account_json,
        secrets: account_secrets_json,
        subaddresses: initial_subaddresses,
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

    let subaddress_indices: HashSet<u64> = txos_and_key_images.iter().map(|(_, _, i)| *i).collect();
    let related_subaddresses: Vec<_> = subaddress_indices
        .iter()
        .map(|i| subaddress_json(&account_key, *i, ""))
        .collect();

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
        subaddresses: related_subaddresses,
    };

    // Write result to file.
    let filename = format!("{}_completed.json", sync_request.trim_end_matches(".json"));
    write_json_command_request_to_file(&json_command_request, &filename);
}

fn generate_subaddresses(secret_mnemonic: &str, request: &str) {
    // Load account key.
    let mnemonic_json =
        fs::read_to_string(secret_mnemonic).expect("Could not open secret mnemonic file.");
    let account_secrets: AccountSecrets = serde_json::from_str(&mnemonic_json).unwrap();
    let account_key = account_key_from_mnemonic_phrase(&account_secrets.mnemonic.unwrap());

    // Load input txos.
    let request_data =
        fs::read_to_string(request).expect("Could not open generate subaddresses request file.");
    let request_json: serde_json::Value =
        serde_json::from_str(&request_data).expect("Malformed generate subaddresses request.");
    let account_id = request_json.get("account_id").unwrap().as_str().unwrap();
    assert_eq!(account_secrets.account_id, account_id);

    let next_subaddress_index = request_json
        .get("next_subaddress_index")
        .unwrap()
        .as_str()
        .unwrap()
        .parse::<u64>()
        .unwrap();

    let num_subaddresses_to_generate = request_json
        .get("num_subaddresses_to_generate")
        .unwrap()
        .as_str()
        .unwrap()
        .parse::<u64>()
        .unwrap();

    let mut subaddresses: Vec<ViewOnlySubaddressJSON> = Vec::new();
    for i in next_subaddress_index..next_subaddress_index + num_subaddresses_to_generate {
        subaddresses.push(subaddress_json(&account_key, i, ""));
    }

    let json_command_request = JsonCommandRequest::import_subaddresses_to_view_only_account {
        account_id: account_id.to_string(),
        subaddresses,
    };
    let filename = format!("{}_completed.json", request.trim_end_matches(".json"));
    write_json_command_request_to_file(&json_command_request, &filename);
}

fn sign_transaction(secret_mnemonic: &str, request: &str) {
    // Load account key.
    let mnemonic_json =
        fs::read_to_string(secret_mnemonic).expect("Could not open secret mnemonic file.");
    let account_secrets: AccountSecrets = serde_json::from_str(&mnemonic_json).unwrap();
    let account_key = account_key_from_mnemonic_phrase(&account_secrets.mnemonic.unwrap());

    // Load input txos.
    let request_data =
        fs::read_to_string(request).expect("Could not open generate subaddresses request file.");
    let request_json: serde_json::Value =
        serde_json::from_str(&request_data).expect("Malformed generate subaddresses request.");
    let account_id = request_json.get("account_id").unwrap().as_str().unwrap();
    assert_eq!(account_secrets.account_id, account_id);

    let unsigned_tx: UnsignedTx = serde_json::from_value(
        request_json
            .get("unsigned_tx")
            .expect("Could not find \"unsigned_tx\".")
            .clone(),
    )
    .unwrap();

    let fog_resolver: FullServiceFogResolver = serde_json::from_value(
        request_json
            .get("fog_resolver")
            .expect("Could not find \"fog_resolver\".")
            .clone(),
    )
    .unwrap();

    let tx_proposal = unsigned_tx.sign(&account_key, fog_resolver);
    let tx_proposal_json = TxProposal::from(&tx_proposal);
    let json_command_request = JsonCommandRequest::submit_transaction {
        tx_proposal: tx_proposal_json,
        comment: None,
        account_id: Some(account_id.to_string()),
    };

    let filename = format!("{}_completed.json", request.trim_end_matches(".json"));
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
    println!("Wrote {}", filename);
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
    Slip10Key::from(mnemonic)
        .try_into_account_key("", "", &base64::decode("").unwrap())
        .unwrap()
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

    match tx_out.amount.get_value(&shared_secret) {
        Ok((_, _)) => true,
        Err(AmountError::InconsistentCommitment) => false,
    }
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

fn subaddress_json(account_key: &AccountKey, index: u64, comment: &str) -> ViewOnlySubaddressJSON {
    let account_id = AccountID::from(account_key);
    let subaddress = account_key.subaddress(index);
    ViewOnlySubaddressJSON {
        object: "view_only_subaddress".to_string(),
        public_address: b58::b58_encode_public_address(&subaddress).unwrap(),
        account_id: account_id.to_string(),
        comment: comment.to_string(),
        subaddress_index: index.to_string(),
        public_spend_key: hex::encode(mc_util_serial::encode(subaddress.spend_public_key())),
    }
}
