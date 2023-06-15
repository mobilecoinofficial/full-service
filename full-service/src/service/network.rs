use crate::db::WalletDbError;
use base64;
use ed25519_dalek::{PublicKey, Signature, Verifier};
use std::error::Error;

const META_DATA_URL: &str = "https://config.mobilecoin.foundation/token_metadata.json";
const SIGNATURE_URL: &str = "https://config.mobilecoin.foundation/token_metadata.sig";
const VERIFIER_KEY: &str = "MCowBQYDK2VwAyEA6rqMXns4wNN+W16Eblsue+gqeXlW5C5WhN3MGCc1Ntw=";

pub struct TokenMetadata {
    pub verified: bool,
    pub metadata: String,
}

pub fn get_token_metadata() -> Result<TokenMetadata, WalletDbError> {
    let metadata = reqwest::blocking::get(META_DATA_URL)?.text()?;
    let message = metadata.as_bytes();
    let mut verified = false;

    let sig = reqwest::blocking::get(SIGNATURE_URL)?.text()?;
    if let Ok(sig) = Signature::from_bytes(sig.as_bytes()) {
        let der_bytes = base64::decode(VERIFIER_KEY)?;
        let der_bytes = der_bytes.as_slice();
        let public_key = PublicKey::from_bytes(der_bytes)?;

        verified = public_key.verify(message, &sig).is_ok();
    }
    Ok(TokenMetadata { verified, metadata })
}
