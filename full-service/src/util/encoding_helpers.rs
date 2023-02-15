use mc_crypto_keys::{RistrettoPrivate, RistrettoPublic};

pub fn ristretto_to_vec(key: &RistrettoPrivate) -> Vec<u8> {
    mc_util_serial::encode(key)
}

pub fn ristretto_public_to_vec(key: &RistrettoPublic) -> Vec<u8> {
    mc_util_serial::encode(key)
}

pub fn vec_to_hex(key: &[u8]) -> String {
    hex::encode(key)
}

pub fn hex_to_vec(key: &str) -> Result<Vec<u8>, String> {
    hex::decode(key).map_err(|err| format!("Could not decode string to vector: {err:?}"))
}

pub fn vec_to_ristretto(key: &[u8]) -> Result<RistrettoPrivate, String> {
    mc_util_serial::decode(key)
        .map_err(|err| format!("Could not decode vector to ristretto: {err:?}"))
}

pub fn vec_to_ristretto_public(key: &[u8]) -> Result<RistrettoPublic, String> {
    mc_util_serial::decode(key)
        .map_err(|err| format!("Could not decode vector to ristretto public: {err:?}"))
}

pub fn hex_to_ristretto(key: &str) -> Result<RistrettoPrivate, String> {
    vec_to_ristretto(&hex_to_vec(key)?)
}

pub fn hex_to_ristretto_public(key: &str) -> Result<RistrettoPublic, String> {
    vec_to_ristretto_public(&hex_to_vec(key)?)
}

pub fn ristretto_to_hex(key: &RistrettoPrivate) -> String {
    vec_to_hex(&ristretto_to_vec(key))
}

pub fn ristretto_public_to_hex(key: &RistrettoPublic) -> String {
    vec_to_hex(&ristretto_public_to_vec(key))
}
