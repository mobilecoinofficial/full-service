use crate::{
    test_utils::create_test_txo_for_recipient,
    util::b58::{
        b58_decode_payment_request, b58_decode_public_address, b58_decode_transfer_payload,
        b58_encode_payment_request, b58_encode_public_address, b58_encode_transfer_payload,
        b58_printable_wrapper_type, B58Error, PrintableWrapperType,
    },
};
use bip39::{Language, Mnemonic};
use mc_account_keys::{AccountKey, PublicAddress};
use mc_core::slip10::Slip10KeyGenerator;
use mc_transaction_core::{tokens::Mob, Amount, Token};
use rand::{rngs::StdRng, CryptoRng, RngCore, SeedableRng};

fn get_public_address<T: RngCore + CryptoRng>(rng: &mut T) -> PublicAddress {
    let account_key = AccountKey::random(rng);
    account_key.default_subaddress()
}

fn get_account_and_entropy_bytes() -> (AccountKey, Vec<u8>) {
    let mnemonic = Mnemonic::from_phrase("rabbit private camp deliver word waste skill loan giant spirit auction brief possible defense spin combine butter satisfy cruise capital depth oval trim inch", Language::English).unwrap();
    let bip39_entropy_bytes = mnemonic.entropy().to_vec();
    let slip10_key = mnemonic.derive_slip10_key(0);
    let account_key = AccountKey::from(slip10_key);
    (account_key, bip39_entropy_bytes)
}

#[test]
/// Encoding a valid PublicAddress should return Ok.
fn encoding_public_address_succeeds() {
    // TODO: this should use property-based testing to generate random
    // public_addresses.
    let mut rng: StdRng = SeedableRng::from_seed([91u8; 32]);
    let public_address = get_public_address(&mut rng);

    let _encoded = b58_encode_public_address(&public_address).unwrap();
}

#[test]
fn encoding_payment_request_succeeds() {
    let mut rng: StdRng = SeedableRng::from_seed([91u8; 32]);
    let public_address = get_public_address(&mut rng);
    let _encoded = b58_encode_payment_request(
        &public_address,
        &Amount::new(1_000_000_000_000, Mob::ID),
        "This is a memo".to_string(),
    )
    .unwrap();
}

#[test]
fn encoding_transfer_payload_succeeds() {
    let mut rng: StdRng = SeedableRng::from_seed([91u8; 32]);
    let (account_key, bip39_entropy_bytes) = get_account_and_entropy_bytes();

    let (txo, _key_image) = create_test_txo_for_recipient(
        &account_key,
        0,
        Amount::new(1_000_000_000_000, Mob::ID),
        &mut rng,
    );

    let proto_tx_pubkey: mc_api::external::CompressedRistretto = (&txo.public_key).into();

    let _encoded = b58_encode_transfer_payload(
        bip39_entropy_bytes,
        proto_tx_pubkey,
        "test transfer payload".to_string(),
    )
    .unwrap();
}

#[test]
/// Decoding a valid b58 string should return the correct PublicAddress.
fn decoding_public_address_succeeds() {
    // TODO: this should use property-based testing to generate random
    // public_addresses.
    let mut rng: StdRng = SeedableRng::from_seed([91u8; 32]);
    let public_address = get_public_address(&mut rng);
    let encoded = b58_encode_public_address(&public_address).unwrap();
    let decoded = b58_decode_public_address(&encoded).unwrap();
    assert_eq!(public_address, decoded);
}

#[test]
fn decoding_payment_request_succeeds() {
    let mut rng: StdRng = SeedableRng::from_seed([91u8; 32]);
    let public_address = get_public_address(&mut rng);
    let encoded = b58_encode_payment_request(
        &public_address,
        &Amount::new(1_000_000_000_000, Mob::ID),
        "This is a memo".to_string(),
    )
    .unwrap();
    let decoded = b58_decode_payment_request(encoded).unwrap();

    assert_eq!(decoded.public_address, public_address);
    assert_eq!(decoded.value, 1_000_000_000_000);
    assert_eq!(decoded.token_id, Mob::ID);
    assert_eq!(decoded.memo, "This is a memo".to_string());
}

#[test]
fn decoding_transfer_payload_succeeds() {
    let mut rng: StdRng = SeedableRng::from_seed([91u8; 32]);
    let (account_key, bip39_entropy_bytes) = get_account_and_entropy_bytes();

    let (txo, _key_image) = create_test_txo_for_recipient(
        &account_key,
        0,
        Amount::new(1_000_000_000_000, Mob::ID),
        &mut rng,
    );

    let proto_tx_pubkey: mc_api::external::CompressedRistretto = (&txo.public_key).into();

    let encoded = b58_encode_transfer_payload(
        bip39_entropy_bytes,
        proto_tx_pubkey,
        "test transfer payload".to_string(),
    )
    .unwrap();

    let decoded = b58_decode_transfer_payload(encoded).unwrap();
    assert_eq!(decoded.account_key, account_key);
    assert_eq!(decoded.txo_public_key, txo.public_key);
    assert_eq!(decoded.memo, "test transfer payload".to_string());
}

#[test]
fn decoding_invalid_public_address_returns_error() {
    let mut rng: StdRng = SeedableRng::from_seed([91u8; 32]);
    let (account_key, bip39_entropy_bytes) = get_account_and_entropy_bytes();

    let (txo, _key_image) = create_test_txo_for_recipient(
        &account_key,
        0,
        Amount::new(1_000_000_000_000, Mob::ID),
        &mut rng,
    );

    let proto_tx_pubkey: mc_api::external::CompressedRistretto = (&txo.public_key).into();

    let encoded = b58_encode_transfer_payload(
        bip39_entropy_bytes,
        proto_tx_pubkey,
        "test transfer payload".to_string(),
    )
    .unwrap();

    let error_type = b58_decode_public_address(&encoded).err();
    assert_eq!(error_type, Some(B58Error::NotPublicAddress));
}

#[test]
fn decoding_invalid_payment_request_returns_error() {
    let mut rng: StdRng = SeedableRng::from_seed([91u8; 32]);
    let public_address = get_public_address(&mut rng);
    let encoded = b58_encode_public_address(&public_address).unwrap();

    let error_type = b58_decode_payment_request(encoded).err();
    assert_eq!(error_type, Some(B58Error::NotPaymentRequest));
}

#[test]
fn decoding_invalid_transfer_payload_returns_error() {
    let mut rng: StdRng = SeedableRng::from_seed([91u8; 32]);
    let public_address = get_public_address(&mut rng);
    let encoded = b58_encode_public_address(&public_address).unwrap();

    let error_type = b58_decode_transfer_payload(encoded).err();

    assert_eq!(error_type, Some(B58Error::NotTransferPayload));
}

#[test]
fn check_public_address_printable_wrapper_type_returns_correct() {
    let mut rng: StdRng = SeedableRng::from_seed([91u8; 32]);
    let public_address = get_public_address(&mut rng);
    let encoded = b58_encode_public_address(&public_address).unwrap();
    let b58_type = b58_printable_wrapper_type(encoded).unwrap();

    assert_eq!(b58_type, PrintableWrapperType::PublicAddress);
}

#[test]
fn check_payment_request_printable_wrapper_type_returns_correct() {
    let mut rng: StdRng = SeedableRng::from_seed([91u8; 32]);
    let public_address = get_public_address(&mut rng);
    let encoded = b58_encode_payment_request(
        &public_address,
        &Amount::new(1_000_000_000_000, Mob::ID),
        "This is a memo".to_string(),
    )
    .unwrap();
    let b58_type = b58_printable_wrapper_type(encoded).unwrap();

    assert_eq!(b58_type, PrintableWrapperType::PaymentRequest);
}

#[test]
fn check_transfer_payload_printable_wrapper_type_returns_correct() {
    let mut rng: StdRng = SeedableRng::from_seed([91u8; 32]);
    let (account_key, bip39_entropy_bytes) = get_account_and_entropy_bytes();
    let (txo, _key_image) = create_test_txo_for_recipient(
        &account_key,
        0,
        Amount::new(1_000_000_000_000, Mob::ID),
        &mut rng,
    );

    let proto_tx_pubkey: mc_api::external::CompressedRistretto = (&txo.public_key).into();

    let encoded = b58_encode_transfer_payload(
        bip39_entropy_bytes,
        proto_tx_pubkey,
        "test transfer payload".to_string(),
    )
    .unwrap();

    let b58_type = b58_printable_wrapper_type(encoded).unwrap();

    assert_eq!(b58_type, PrintableWrapperType::TransferPayload);
}

#[test]
/// Attempting to decode invalid data should return a reasonable Error.
fn decoding_insufficient_bytes_string() {
    let invalid_b58_string = "1234";

    let error_type = b58_decode_public_address(invalid_b58_string).err();
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
