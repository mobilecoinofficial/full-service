
use structopt::StructOpt;
use transaction_signer_lib
    ::{create_account, generate_view_only_import_package, sign_transaction, sync_txos};

#[derive(Clone, Debug, StructOpt)]
#[structopt(
    name = "transaction-signer",
    about = "MobileCoin offline transaction signer"
)]
enum Opts {
    /// Generate an account, save the mnemonic and the request to import the view only account. 
    Create {
        #[structopt(short, long)]
        name: Option<String>,
    },
    /// Import an account, save the nmemonic and the request to import the view only account.
    Import {
        #[structopt(short, long)]
        name: Option<String>,
        mnemonic: String,
    },
    /// Sync txos with a mnemonic and a sync request.
    r#Sync {
        secret_mnemonic: String,
        sync_request: String,
        #[structopt(short, long, default_value = "1000")]
        subaddresses: u64,
    },
    /// Sign a transaction and save the request to submit the transaction. 
    Sign {
        secret_mnemonic: String,
        request: String,
    },
    /// Generate a request to import a view-only account from a secret mnemonic.
    ViewOnlyImportPackage {
        secret_mnemonic: String,
    },
}
pub fn main() {
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