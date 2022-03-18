// Copyright (c) 2020-2021 MobileCoin Inc.

//! API definition for the Account Key object.

use crate::util::encoding_helpers::{hex_to_ristretto, hex_to_vec, ristretto_to_hex, vec_to_hex};
use serde_derive::{Deserialize, Serialize};
use std::convert::TryFrom;

/// The AccountKey contains a View keypair and a Spend keypair, used to
/// construct and receive transactions.
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct AccountKey {
    /// String representing the object's type. Objects of the same type share
    /// the same value.
    pub object: String,

    ///  Private key used for view-key matching, hex-encoded Ristretto bytes.
    pub view_private_key: String,

    /// Private key used for spending, hex-encoded Ristretto bytes.
    pub spend_private_key: String,

    /// Fog Report server url (if user has Fog service), empty string otherwise.
    pub fog_report_url: String,

    /// Fog Report Key (if user has Fog service), empty otherwise
    /// The key labelling the report to use, from among the several reports
    /// which might be served by the fog report server.
    pub fog_report_id: String,

    /// Fog Authority Subject Public Key Info (if user has Fog service),
    /// empty string otherwise.
    pub fog_authority_spki: String,
}

impl From<&mc_account_keys::AccountKey> for AccountKey {
    fn from(src: &mc_account_keys::AccountKey) -> AccountKey {
        AccountKey {
            object: "account_key".to_string(),
            view_private_key: ristretto_to_hex(src.view_private_key()),
            spend_private_key: ristretto_to_hex(src.spend_private_key()),
            fog_report_url: src.fog_report_url().unwrap_or("").to_string(),
            fog_report_id: src.fog_report_id().unwrap_or("").to_string(),
            fog_authority_spki: vec_to_hex(&src.fog_authority_spki().unwrap_or(&[]).to_vec()),
        }
    }
}

impl TryFrom<&AccountKey> for mc_account_keys::AccountKey {
    type Error = String;

    fn try_from(src: &AccountKey) -> Result<mc_account_keys::AccountKey, String> {
        let view_private_key = hex_to_ristretto(&src.view_private_key)?;
        let spend_private_key = hex_to_ristretto(&src.spend_private_key)?;
        let fog_authority_spki = hex_to_vec(&src.fog_authority_spki)?;

        Ok(mc_account_keys::AccountKey::new_with_fog(
            &spend_private_key,
            &view_private_key,
            src.fog_report_url.clone(),
            src.fog_report_id.clone(),
            fog_authority_spki,
        ))
    }
}

#[cfg(test)]
mod account_key_tests {
    use super::*;
    use rand::{rngs::StdRng, SeedableRng};

    #[test]
    fn test_round_trip() {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let account_key1 = mc_account_keys::AccountKey::random(&mut rng);
        let json_rpc_account_key1 = AccountKey::try_from(&account_key1).unwrap();
        let json_account_key = serde_json::json!(json_rpc_account_key1);

        let json_rpc_account_key2: AccountKey = serde_json::from_value(json_account_key).unwrap();
        let account_key2 = mc_account_keys::AccountKey::try_from(&json_rpc_account_key2).unwrap();

        assert_eq!(account_key1, account_key2);
    }
}
