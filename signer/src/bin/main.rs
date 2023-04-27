// Copyright (c) 2020-2023 MobileCoin Inc.

//! transaction signer implementation

use bip39::{Language, Mnemonic};
use clap::Parser;
use mc_common::logger::global_log;
use mc_core::{account::Account, slip10::Slip10KeyGenerator};
use mc_crypto_ring_signature_signer::LocalRingSigner;
use mc_signer::service;
use mc_transaction_core::AccountKey;
use mc_transaction_signer::{read_input, types::AccountInfo, write_output, Operations};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, PartialEq, Debug, Parser)]
struct Args {
    #[clap(long, short, default_value = "mc_account_secrets.json")]
    account_secrets_file: String,

    #[command(subcommand)]
    action: Actions,
}

#[derive(Clone, PartialEq, Debug, Parser)]
enum Actions {
    /// Create a new offline account, writing secrets to the output file
    Create {
        /// File name for account secrets to be written to
        #[clap(long)]
        output: String,
    },

    // Implement shared signer commands
    #[command(flatten)]
    Signer(Operations),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
struct AccountSecrets {
    mnemonic: String,
    account_info: AccountInfo,
}

fn main() -> anyhow::Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    // Run commands
    match &args.action {
        Actions::Create { output } => {
            let (mnemonic, account_info) = service::create_account();
            let account_secrets = AccountSecrets {
                mnemonic: mnemonic.to_string(),
                account_info,
            };

            // Check we're not overwriting an existing secret file
            if Path::new(output).exists() {
                return Err(anyhow::anyhow!(
                    "creation would overwrite existing secrets file '{}'",
                    output
                ));
            }

            // Otherwise write out new secrets
            write_output(output, &account_secrets)?;

            global_log::info!("Account secrets written to '{}'", output);
        }
        Actions::Signer(operation) => {
            // Load account secrets
            let secrets: AccountSecrets = read_input(&args.account_secrets_file)?;
            let mnemonic = Mnemonic::from_phrase(&secrets.mnemonic, Language::English)?;

            // Perform SLIP-0010 derivation
            let account_index = operation.account_index();
            let slip10key = mnemonic.derive_slip10_key(account_index);

            // Generate account from secrets
            let account: Account = Account::from(&slip10key);

            // Handle standard commands
            match operation {
                Operations::GetAccount { output, .. } => {
                    Operations::get_account(&account, account_index, output)?
                }
                Operations::SyncTxos { input, output, .. } => {
                    Operations::sync_txos(&account, input, output)?
                }
                Operations::SignTx { input, output, .. } => {
                    // Setup local ring signer
                    let ring_signer = LocalRingSigner::from(&AccountKey::new(
                        account.spend_private_key().as_ref(),
                        account.view_private_key().as_ref(),
                    ));

                    // Perform transaction signing
                    Operations::sign_tx(&ring_signer, input, output)?;
                }
                _ => {
                    panic!("This command is not supported, please run 'help' for more information about supported commands.")
                }
            }
        }
    }

    Ok(())
}
