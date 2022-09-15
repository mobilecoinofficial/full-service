use mc_account_keys::PublicAddress;
use mc_common::HashMap;
use mc_crypto_keys::RistrettoPublic;
use mc_fog_report_validation::{FogPubkeyError, FogPubkeyResolver, FullyValidatedFogPubkey};
use serde::{Deserialize, Serialize};

use mc_full_service_utils::b58::b58_encode_public_address;

use std::convert::TryFrom;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FullServiceFogResolver(pub HashMap<String, FullServiceFullyValidatedFogPubkey>);

impl FogPubkeyResolver for FullServiceFogResolver {
    fn get_fog_pubkey(
        &self,
        address: &PublicAddress,
    ) -> Result<FullyValidatedFogPubkey, FogPubkeyError> {
        let b58_address =
            b58_encode_public_address(address).map_err(|_| FogPubkeyError::NoFogReportUrl)?;

        let fs_fog_pubkey = match self.0.get(&b58_address) {
            Some(pubkey) => Ok(pubkey.clone()),
            None => Err(FogPubkeyError::NoFogReportUrl),
        }?;

        FullyValidatedFogPubkey::try_from(fs_fog_pubkey)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FullServiceFullyValidatedFogPubkey {
    pub pubkey: [u8; 32],
    pub pubkey_expiry: u64,
}

impl From<FullyValidatedFogPubkey> for FullServiceFullyValidatedFogPubkey {
    fn from(fog_pubkey: FullyValidatedFogPubkey) -> Self {
        Self {
            pubkey: fog_pubkey.pubkey.to_bytes(),
            pubkey_expiry: fog_pubkey.pubkey_expiry,
        }
    }
}

impl TryFrom<FullServiceFullyValidatedFogPubkey> for FullyValidatedFogPubkey {
    type Error = FogPubkeyError;

    fn try_from(fog_pubkey: FullServiceFullyValidatedFogPubkey) -> Result<Self, Self::Error> {
        Ok(Self {
            pubkey: RistrettoPublic::try_from(&fog_pubkey.pubkey)
                .map_err(|_| FogPubkeyError::NoFogReportUrl)?,
            pubkey_expiry: fog_pubkey.pubkey_expiry,
        })
    }
}
