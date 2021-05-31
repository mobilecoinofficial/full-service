#[cfg(test)]
mod tests {
    use crate::util::b58::{
        b58_decode, b58_decode_payment_request, b58_decode_transfer_payload, b58_encode,
        b58_encode_payment_request, b58_encode_transfer_payload, b58_printable_wrapper_type,
        B58Error, PrintableWrapperType,
    };
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
    fn encoding_payment_request_succeeds() {
        let mut rng: StdRng = SeedableRng::from_seed([91u8; 32]);
        let public_address = get_public_address(&mut rng);
        let _encoded = b58_encode_payment_request(
            &public_address,
            1_000_000_000_000,
            "This is a memo".to_string(),
        )
        .unwrap();
    }

    #[test]
    fn decoding_payment_request_succeeds() {
        let mut rng: StdRng = SeedableRng::from_seed([91u8; 32]);
        let public_address = get_public_address(&mut rng);
        let encoded = b58_encode_payment_request(
            &public_address,
            1_000_000_000_000,
            "This is a memo".to_string(),
        )
        .unwrap();
        let decoded = b58_decode_payment_request(encoded).unwrap();

        assert_eq!(decoded.public_address, public_address);
        assert_eq!(decoded.value, 1_000_000_000_000);
        assert_eq!(decoded.memo, "This is a memo".to_string());
    }

    #[test]
    fn decoding_invalid_payment_request_returns_error() {
        let mut rng: StdRng = SeedableRng::from_seed([91u8; 32]);
        let public_address = get_public_address(&mut rng);
        let encoded = b58_encode(&public_address).unwrap();

        let error_type = b58_decode_payment_request(encoded).err();
        assert_eq!(error_type, Some(B58Error::NotPaymentRequest));
    }

    #[test]
    fn decoding_invalid_transfer_payload_returns_error() {
        let mut rng: StdRng = SeedableRng::from_seed([91u8; 32]);
        let public_address = get_public_address(&mut rng);
        let encoded = b58_encode(&public_address).unwrap();

        let error_type = b58_decode_transfer_payload(encoded).err();

        assert_eq!(error_type, Some(B58Error::NotTransferPayload));
    }

    #[test]
    fn check_public_address_printable_wrapper_type_returns_correct() {
        let mut rng: StdRng = SeedableRng::from_seed([91u8; 32]);
        let public_address = get_public_address(&mut rng);
        let encoded = b58_encode(&public_address).unwrap();
        let b58_type = b58_printable_wrapper_type(encoded).unwrap();

        assert_eq!(b58_type, PrintableWrapperType::PublicAddress);
    }

    #[test]
    fn check_payment_request_printable_wrapper_type_returns_correct() {
        let mut rng: StdRng = SeedableRng::from_seed([91u8; 32]);
        let public_address = get_public_address(&mut rng);
        let encoded = b58_encode_payment_request(
            &public_address,
            1_000_000_000_000,
            "This is a memo".to_string(),
        )
        .unwrap();
        let b58_type = b58_printable_wrapper_type(encoded).unwrap();

        assert_eq!(b58_type, PrintableWrapperType::PaymentRequest);
    }

    #[test]
    #[ignore]
    fn encoding_transfer_payload_succeeds() {
        // TODO
        unimplemented!()
    }

    #[test]
    #[ignore]
    fn decoding_transfer_payload_succeeds() {
        // TODO
        unimplemented!()
    }

    #[test]
    #[ignore]
    fn check_transfer_payload_printable_wrapper_type_returns_correct() {
        unimplemented!()
    }

    #[test]
    /// Attempting to decode invalid data should return a reasonable Error.
    fn decoding_insufficient_bytes_string() {
        let invalid_b58_string = "1234";

        let error_type = b58_decode(invalid_b58_string).err();
        assert_eq!(
            error_type,
            Some(B58Error::PrintableWrapper(
                mc_api::display::Error::InsufficientBytes(3)
            ))
        );

        let error_type = b58_decode_payment_request(invalid_b58_string.to_string()).err();
        assert_eq!(
            error_type,
            Some(B58Error::PrintableWrapper(
                mc_api::display::Error::InsufficientBytes(3)
            ))
        );

        let error_type = b58_decode_transfer_payload(invalid_b58_string.to_string()).err();
        assert_eq!(
            error_type,
            Some(B58Error::PrintableWrapper(
                mc_api::display::Error::InsufficientBytes(3)
            ))
        );
    }
}
