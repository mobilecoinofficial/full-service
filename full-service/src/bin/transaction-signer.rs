use bip39::{Language, Mnemonic, MnemonicType};
use mc_account_keys::{AccountKey, CHANGE_SUBADDRESS_INDEX, DEFAULT_SUBADDRESS_INDEX};
use mc_account_keys_slip10::Slip10Key;
use mc_full_service::db::account::AccountID;
use std::fs;
use structopt::StructOpt;

use mc_full_service::json_rpc::{
    account_key::AccountKey as AccountKeyJSON,
    account_secrets::AccountSecrets,
    view_only_account::{
        ViewOnlyAccountImportPackageJSON, ViewOnlyAccountJSON, ViewOnlyAccountSecretsJSON,
    },
    view_only_subaddress::ViewOnlySubaddressJSON,
};

use mc_full_service::util::b58;

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
}

fn main() {
    let opts = Opts::from_args();

    match opts {
        Opts::Create { ref name } => {
            create_account(name.clone().unwrap_or("".into()));
        }
    }
}

fn create_account(name: String) {
    println!("Creating account {}", name);

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

    let filename = format!(
        "mobilecoin_secret_mnemonic_{}.json",
        &account_id.to_string()[..16]
    );
    let output_json = serde_json::to_string_pretty(&secrets).unwrap();
    fs::write(&filename, output_json + "\n").expect("could not write output file");
    println!("Wrote {}", filename);

    let import_package = _create_import_account_package(name, &account_key);
    let filename = format!(
        "mobilecoin_view_account_import_package_{}.json",
        &account_id.to_string()[..16]
    );
    let output_json = serde_json::to_string_pretty(&import_package).unwrap();
    fs::write(&filename, output_json + "\n").expect("could not write output file");
    println!("Wrote {}", filename);
}

fn _create_import_account_package(
    name: String,
    account_key: &AccountKey,
) -> ViewOnlyAccountImportPackageJSON {
    let account_id = AccountID::from(account_key);

    let account_json = ViewOnlyAccountJSON {
        object: "view_only_account".to_string(),
        name,
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

    ViewOnlyAccountImportPackageJSON {
        object: "view_only_account_import_package".to_string(),
        account: account_json,
        secrets: account_secrets_json,
        subaddresses: subaddresses_json,
    }
}
