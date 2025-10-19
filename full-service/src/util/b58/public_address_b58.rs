//! Serde serialization/deserialization helpers for B58 encoding.

use super::{b58_decode_public_address, b58_encode_public_address};
use mc_account_keys::PublicAddress;
use serde::{Deserialize, Deserializer, Serializer};

pub fn serialize<S>(public_address: &PublicAddress, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let b58_string =
        b58_encode_public_address(public_address).map_err(serde::ser::Error::custom)?;
    serializer.serialize_str(&b58_string)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<PublicAddress, D::Error>
where
    D: Deserializer<'de>,
{
    let b58_string = <String>::deserialize(deserializer)?;
    b58_decode_public_address(&b58_string).map_err(serde::de::Error::custom)
}

pub fn deserialize_opt<'de, D>(deserializer: D) -> Result<Option<PublicAddress>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt {
        Some(s) if !s.is_empty() => b58_decode_public_address(&s).map(Some).map_err(serde::de::Error::custom),
        _ => Ok(None),
    }
}
