use crate::db::WalletDbError;
use base64::{engine::general_purpose, Engine};
use ed25519_dalek::{Signature, Verifier, VerifyingKey, PUBLIC_KEY_LENGTH};

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

    if let Ok(sig) = Signature::from_slice(sig.as_bytes()) {
        let mut der_slice = [0u8; PUBLIC_KEY_LENGTH];
        general_purpose::STANDARD.decode_slice(VERIFIER_KEY, &mut der_slice)?;
        let public_key = VerifyingKey::from_bytes(&der_slice)?;

        verified = public_key.verify(message, &sig).is_ok();
    }
    Ok(TokenMetadata { verified, metadata })
}
