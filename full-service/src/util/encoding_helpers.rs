use mc_crypto_keys::RistrettoPrivate;

// Full service used to store certain data in protobuff format. As of (TODO(cc)
// date), it stores the actual data instead. This function supports
// installlations of full service that were running prior to removing protobuff
// data.
// pub fn decode_if_protobuff(data: &[u8]) -> Result<&[u8],
// mc_util_serial::DecodeError> {     match mc_util_serial::decode(data) {
//         Result::Ok(decoded_protobuff) => decoded_protobuff,
//         Result::Err(_) => Ok(data),
//     }
// }

// TODO(CC) Delete all of these once protobuff data storage removed from app
pub fn ristretto_to_vec(key: &RistrettoPrivate) -> Vec<u8> {
    mc_util_serial::encode(key)
}

pub fn vec_to_hex(key: &[u8]) -> String {
    hex::encode(key)
}

pub fn hex_to_vec(key: &str) -> Result<Vec<u8>, String> {
    hex::decode(key).map_err(|err| format!("Could not decode string to vector: {:?}", err))
}

pub fn vec_to_ristretto(key: &[u8]) -> Result<RistrettoPrivate, String> {
    mc_util_serial::decode(key)
        .map_err(|err| format!("Could not decode vector to ristretto: {:?}", err))
}

pub fn hex_to_ristretto(key: &str) -> Result<RistrettoPrivate, String> {
    vec_to_ristretto(&hex_to_vec(key)?)
}

pub fn ristretto_to_hex(key: &RistrettoPrivate) -> String {
    vec_to_hex(&ristretto_to_vec(key))
}

// mod tests {
//     use super::*;
//     use mc_common::logger::{test_with_logger, Logger};
//     use mc_crypto_keys::RistrettoPrivate;
//     use mc_util_from_random::FromRandom;
//     use rand::{rngs::StdRng, SeedableRng};

//     #[test_with_logger]
//     fn test_decode_if_protobuff(logger: Logger) {
//         let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);
//         let random_key = RistrettoPrivate::from_random(&mut rng);
//         let encoded_key = mc_util_serial::encode(&random_key);
//         let decoded_protobuff: RistrettoPrivate =
// decode_if_protobuff(&encoded_key).unwrap();         let decoded_bytes:
// RistrettoPrivate = decode_if_protobuff(&random_key.to_bytes()).unwrap();

//         assert_eq!(decoded_protobuff.to_bytes(), random_key.to_bytes());
//         assert_eq!(decoded_bytes.to_bytes(), random_key.to_bytes());
//     }
// }
