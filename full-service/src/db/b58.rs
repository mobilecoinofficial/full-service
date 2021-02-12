//! Public Address base58 encoding and decoding.

use crate::db::WalletDbError;
use mc_account_keys::PublicAddress;
use std::convert::TryFrom;

pub fn b58_encode(public_address: &PublicAddress) -> Result<String, WalletDbError> {
    let mut wrapper = mc_mobilecoind_api::printable::PrintableWrapper::new();
    wrapper.set_public_address(public_address.into());
    Ok(wrapper.b58_encode()?)
}

pub fn b58_decode(b58_public_address: &str) -> Result<PublicAddress, WalletDbError> {
    let wrapper =
        mc_mobilecoind_api::printable::PrintableWrapper::b58_decode(b58_public_address.to_string())?;

    let pubaddr_proto: &mc_api::external::PublicAddress = if wrapper.has_payment_request() {
        let payment_request = wrapper.get_payment_request();
        payment_request.get_public_address()
    } else if wrapper.has_public_address() {
        wrapper.get_public_address()
    } else {
        return Err(WalletDbError::B58Decode);
    };

    let public_address = PublicAddress::try_from(pubaddr_proto)
        .map_err(|_e| WalletDbError::B58Decode)?;
    Ok(public_address)
}

#[cfg(test)]
mod tests {
    use mc_account_keys::{PublicAddress, AccountKey};
    use rand::rngs::StdRng;
    use rand::{SeedableRng, RngCore, CryptoRng};
    use crate::db::{b58_encode, b58_decode};

    fn get_public_address<T: RngCore + CryptoRng>(rng: &mut T) -> PublicAddress {
        let account_key = AccountKey::random(rng);
        account_key.default_subaddress()
    }

    #[test]
    /// Encoding a valid PublicAddress should return Ok.
    fn encoding_does_not_panic() {
        // TODO: this should use property-based testing to generate random public_addresses.
        let mut rng: StdRng = SeedableRng::from_seed([91u8; 32]);
        let public_address = get_public_address(&mut rng);

        let _encoded = b58_encode(&public_address).unwrap();
    }

    #[test]
    #[ignore]
    /// Encoded string should be valid b58.
    fn encoding_produces_b58() {
        // TODO
        unimplemented!()
    }

    #[test]
    /// Decoding a valid b58 string should return the correct PublicAddress.
    fn decoding_succeeds() {
        // TODO: this should use property-based testing to generate random public_addresses.
        let mut rng: StdRng = SeedableRng::from_seed([91u8; 32]);
        let public_address = get_public_address(&mut rng);
        let encoded = b58_encode(&public_address).unwrap();
        let decoded = b58_decode(&encoded).unwrap();
        assert_eq!(public_address, decoded);
    }


    #[test]
    #[ignore]
    /// Attempting to decode invalid data should return a reasonable Error.
    fn decoding_invalid_string_should_not_panic() {
        // TODO
        unimplemented!()
    }


}