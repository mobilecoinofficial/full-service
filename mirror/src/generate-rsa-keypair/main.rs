// Copyright (c) 2018-2022 MobileCoin Inc.

//! Utility to generate a 4096-bit passphrase-less RSA keypair, meant to be used for private<->client end to end encryption.

use boring::rsa::Rsa;
use std::{fs, path::Path};

const PRIVATE_KEY_FILENAME: &str = "mirror-private.pem";
const PUBLIC_KEY_FILENAME: &str = "mirror-client.pem";

fn main() {
    if Path::new(PUBLIC_KEY_FILENAME).exists() {
        panic!("{} already exists", PUBLIC_KEY_FILENAME);
    }
    let priv_key = if Path::new(PRIVATE_KEY_FILENAME).exists() {
        println!("Reading existing private key file {}", PRIVATE_KEY_FILENAME);
        let key_str = std::fs::read_to_string(PRIVATE_KEY_FILENAME).unwrap_or_else(|err| {
            panic!(
                "failed reading private key file {}: {}",
                PRIVATE_KEY_FILENAME, err
            )
        });
        Rsa::private_key_from_pem_passphrase(key_str.as_bytes(), &[]).unwrap_or_else(|err| {
            panic!(
                "failed parsing private key file {}: {}",
                PRIVATE_KEY_FILENAME, err
            )
        })
    } else {
        println!("Generating private key, this might take a few seconds...");
        Rsa::generate(4096).expect("failed generating private key")
    };

    let priv_key_pem = priv_key
        .private_key_to_pem()
        .expect("Failed getting privte key as PEM");
    let pub_key_pem = priv_key
        .public_key_to_pem()
        .expect("Failed getting public key as PEM");

    fs::write(PRIVATE_KEY_FILENAME, priv_key_pem).expect("Failed writing private key to file");
    println!("Wrote {} - use this file with the private side of the mirror. See README.md for more details'", PRIVATE_KEY_FILENAME);

    fs::write(PUBLIC_KEY_FILENAME, pub_key_pem).expect("Failed writing public key to file");
    println!(
        "Wrote {}  - use this file with client, see example-client.js for example",
        PUBLIC_KEY_FILENAME
    );
}
