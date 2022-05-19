use bip39::{Language, Mnemonic, MnemonicType};
use mc_account_keys::{CHANGE_SUBADDRESS_INDEX, DEFAULT_SUBADDRESS_INDEX};
use mc_account_keys_slip10::Slip10Key;
use mc_full_service::{
    db::account::AccountID,
    json_rpc::{
        account_key::AccountKey as AccountKeyJSON,
        account_secrets::AccountSecrets,
        view_only_account::{
            ViewOnlyAccountImportPackageJSON, ViewOnlyAccountJSON, ViewOnlyAccountSecretsJSON,
        },
        view_only_subaddress::ViewOnlySubaddressJSON,
    },
    util::b58,
};
use std::fs;
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
    r#Sync {
        secret_mnemonic: String,
        sync_request: String,
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
        } => {
            sync_txos(secret_mnemonic, sync_request);
        }
    }
}

fn create_account(name: &String) {
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
        name: name.clone(),
    };

    // Package view private key.
    let account_json = ViewOnlyAccountJSON {
        object: "view_only_account".to_string(),
        name: name.clone(),
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
    let mut subaddresses_json = Vec::new();

    let main_subaddress = account_key.default_subaddress();
    let change_subaddress = account_key.change_subaddress();

    let main_subaddress_json = ViewOnlySubaddressJSON {
        object: "view_only_subaddress".to_string(),
        public_address: b58::b58_encode_public_address(&main_subaddress).unwrap(),
        account_id: account_id.to_string(),
        comment: "Main".to_string(),
        subaddress_index: DEFAULT_SUBADDRESS_INDEX.to_string(),
        public_spend_key: hex::encode(mc_util_serial::encode(main_subaddress.spend_public_key())),
    };

    let change_subaddress_json = ViewOnlySubaddressJSON {
        object: "view_only_subaddress".to_string(),
        public_address: b58::b58_encode_public_address(&change_subaddress).unwrap(),
        account_id: account_id.to_string(),
        comment: "Change".to_string(),
        subaddress_index: CHANGE_SUBADDRESS_INDEX.to_string(),
        public_spend_key: hex::encode(mc_util_serial::encode(change_subaddress.spend_public_key())),
    };

    subaddresses_json.push(main_subaddress_json);
    subaddresses_json.push(change_subaddress_json);

    // Assemble view-only import package.
    let import_package = ViewOnlyAccountImportPackageJSON {
        object: "view_only_account_import_package".to_string(),
        account: account_json,
        secrets: account_secrets_json,
        subaddresses: subaddresses_json,
    };

    // Write secret mnemonic to file.
    let filename = format!(
        "mobilecoin_secret_mnemonic_{}.json",
        &account_id.to_string()[..6]
    );
    let output_json = serde_json::to_string_pretty(&secrets).unwrap();
    fs::write(&filename, output_json + "\n").expect("could not write output file");
    println!("Wrote {}", filename);

    // Write view private key and associated info to file.
    let filename = format!(
        "mobilecoin_view_account_import_package_{}.json",
        &account_id.to_string()[..6]
    );
    let output_json = serde_json::to_string_pretty(&import_package).unwrap();
    fs::write(&filename, output_json + "\n").expect("could not write output file");
    println!("Wrote {}", filename);
}

fn sync_txos(secret_mnemonic: &String, sync_request: &String) {
    let mnemonic_json =
        fs::read_to_string(secret_mnemonic).expect("Could not open secret mnemonic file.");
    let account_secrets: AccountSecrets = serde_json::from_str(&mnemonic_json).unwrap();
    dbg!(&account_secrets);

    let sync_request_json =
        fs::read_to_string(sync_request).expect("Could not open sync request file.");
    let sync_request: serde_json::Value =
        serde_json::from_str(&sync_request_json).expect("malformed sync request");
    dbg!(&sync_request);
    assert_eq!(
        account_secrets.account_id,
        sync_request.get("account_id").unwrap().as_str().unwrap()
    );

    // let input_txos = sync_request.get("

    // let input_txos_serialized: Vec<Vec<u8>> =
    // serde_json::from_str(&input_txos_json).unwrap(); let input_txos:
    // Vec<TxOut> = input_txos_serialized.iter()
    //     .map(|tx_out_serialized| {
    //         let tx_out: TxOut =
    // mc_util_serial::decode(tx_out_serialized).unwrap();         tx_out
    //     })
    //     .collect();

    // let serialized_txos_and_key_images =
    //     _get_key_images_for_txos(&input_txos, account_key,
    // subaddress_spend_public_keys);
    // let serialized_txos_and_key_images_data =
    //     serde_json::to_string(&serialized_txos_and_key_images).unwrap();
}

// fn _get_key_images_for_txos(
//     tx_outs: &[TxOut],
//     account_key: &AccountKey,
//     subaddress_spend_public_keys: &HashMap<RistrettoPublic, u64>,
// ) -> Vec<(Vec<u8>, Vec<u8>)> {
//     let mut serialized_txos_and_key_images: Vec<(Vec<u8>, Vec<u8>)> =
// Vec::new();

//     for tx_out in tx_outs.iter() {
//         if tx_out_belongs_to_account(tx_out, account_key.view_private_key())
// {             if let Some(key_image) =
//                 get_key_image_for_tx_out(tx_out, account_key,
// subaddress_spend_public_keys)             {
//                 serialized_txos_and_key_images.push((
//                     mc_util_serial::encode(tx_out),
//                     mc_util_serial::encode(&key_image),
//                 ));
//             }
//         }
//     }

//     serialized_txos_and_key_images
// }
