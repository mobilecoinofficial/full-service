#[cfg(test)]
mod tests {
    use crate::util::b58::{b58_decode, b58_encode};
    use mc_account_keys::{AccountKey, PublicAddress};
    use rand::{rngs::StdRng, CryptoRng, RngCore, SeedableRng};

    fn get_public_address<T: RngCore + CryptoRng>(rng: &mut T) -> PublicAddress {
        let account_key = AccountKey::random(rng);
        account_key.default_subaddress()
    }

    #[test]
    /// Encoding a valid PublicAddress should return Ok.
    fn encoding_does_not_panic() {
        // TODO: this should use property-based testing to generate random
        // public_addresses.
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
        // TODO: this should use property-based testing to generate random
        // public_addresses.
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
