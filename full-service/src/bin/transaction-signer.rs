use structopt::StructOpt;

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

fn create_account(name: String) {
    println!("Creating account {}", name);
}
