use structopt::StructOpt;
use mc_account_keys_slip10::Slip10Key;
use mc_full_service::db::account::AccountID;
use bip39::{Language, Mnemonic, MnemonicType};
use serde::{Deserialize, Serialize};
use std::fs;

/// Command line config for the Wallet API


#[derive(Clone, Debug, StructOpt)]
#[structopt(name = "transaction-signer", about = "MobileCoin offline transaction signer")]
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
        },
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
struct MnemonicPackage {
    account_id: String,
    account_name: String,
    mnemonic: String,
    key_derivation_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ViewKeyPackage {
    view_private_key: String,
}

fn create_account(name: String) {
    println!("Creating account {}", name);

    let mnemonic = Mnemonic::new(MnemonicType::Words24, Language::English);

    let fog_report_url = "".to_string();
    let fog_report_id = "".to_string();
    let fog_authority_spki = "".to_string();
    let account_key = Slip10Key::from(mnemonic.clone()).try_into_account_key(
        &fog_report_url,
        &fog_report_id,
        &base64::decode(fog_authority_spki).expect("Invalid Fog SPKI"),
    ).expect("could not generate account key");
    let account_id = AccountID::from(&account_key);

    let package = MnemonicPackage {
        account_name: name,
        account_id: account_id.to_string(),
        mnemonic: mnemonic.phrase().to_string(),
        key_derivation_version: "2".to_string(),
    };
    let filename = format!("mobilecoin_seed_mnemonic_{}.json", &account_id.to_string()[..16]);
    let output_json = serde_json::to_string_pretty(&package).unwrap();
    fs::write(&filename, output_json + "\n").expect("could not write output file");
    println!("Wrote {}", filename);

}
