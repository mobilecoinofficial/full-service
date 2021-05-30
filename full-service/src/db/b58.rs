//! Public Address base58 encoding and decoding.

use bip39::{Language, Mnemonic};
use displaydoc::Display;
use mc_account_keys::{AccountKey, PublicAddress, RootEntropy, RootIdentity};
use mc_account_keys_slip10::Slip10KeyGenerator;
use mc_crypto_keys::CompressedRistrettoPublic;
use std::convert::TryFrom;

pub struct DecodedPaymentRequest {
    pub public_address: PublicAddress,
    pub value: u64,
    pub memo: String,
}

pub struct DecodedTransferPayload {
    pub root_entropy: Option<RootEntropy>,
    pub bip39_entropy: Option<Vec<u8>>,
    pub account_key: AccountKey,
    pub txo_public_key: CompressedRistrettoPublic,
    pub memo: String,
}

#[derive(Display, Debug)]
pub enum B58Error {
    /// Invalid Entropy
    InvalidEntropy,

    /// Proto Conversion
    ProtoConversion(mc_api::ConversionError),

    /// Printable Wrapper
    PrintableWrapper(mc_api::display::Error),
}

impl From<mc_api::ConversionError> for B58Error {
    fn from(src: mc_api::ConversionError) -> Self {
        Self::ProtoConversion(src)
    }
}

impl From<mc_api::display::Error> for B58Error {
    fn from(src: mc_api::display::Error) -> Self {
        Self::PrintableWrapper(src)
    }
}

pub fn b58_encode(public_address: &PublicAddress) -> Result<String, B58Error> {
    let mut wrapper = mc_mobilecoind_api::printable::PrintableWrapper::new();
    wrapper.set_public_address(public_address.into());
    Ok(wrapper.b58_encode()?)
}

pub fn b58_decode(b58_public_address: &str) -> Result<PublicAddress, B58Error> {
    let wrapper = mc_mobilecoind_api::printable::PrintableWrapper::b58_decode(
        b58_public_address.to_string(),
    )?;

    let pubaddr_proto: &mc_api::external::PublicAddress = if wrapper.has_payment_request() {
        let payment_request = wrapper.get_payment_request();
        payment_request.get_public_address()
    } else if wrapper.has_public_address() {
        wrapper.get_public_address()
    } else {
        return Err(B58Error::InvalidEntropy);
    };

    let public_address = PublicAddress::try_from(pubaddr_proto)?;
    Ok(public_address)
}

pub fn b58_encode_payment_request(
    public_address: &PublicAddress,
    amount_pmob: u64,
    memo: String,
) -> Result<String, B58Error> {
    let mut payment_request = mc_mobilecoind_api::printable::PaymentRequest::new();
    payment_request.set_public_address(public_address.into());
    payment_request.set_value(amount_pmob);
    payment_request.set_memo(memo);

    let mut wrapper = mc_mobilecoind_api::printable::PrintableWrapper::new();
    wrapper.set_payment_request(payment_request);

    Ok(wrapper.b58_encode()?)
}

pub fn b58_decode_payment_request(
    payment_request_b58: String,
) -> Result<DecodedPaymentRequest, B58Error> {
    let wrapper = mc_mobilecoind_api::printable::PrintableWrapper::b58_decode(
        payment_request_b58.to_string(),
    )?;
    let payment_request_message = if wrapper.has_payment_request() {
        wrapper.get_payment_request()
    } else {
        return Err(B58Error::InvalidEntropy);
    };

    let public_address = PublicAddress::try_from(payment_request_message.get_public_address())?;
    let value = payment_request_message.get_value() as u64;
    let memo = payment_request_message.get_memo().to_string();

    Ok(DecodedPaymentRequest {
        public_address,
        value,
        memo,
    })
}

pub fn b58_encode_transfer_payload(
    bip_39_entropy_bytes: Vec<u8>,
    proto_tx_pubkey: mc_api::external::CompressedRistretto,
    memo: String,
) -> Result<String, B58Error> {
    let mut transfer_payload = mc_mobilecoind_api::printable::TransferPayload::new();
    transfer_payload.set_bip39_entropy(bip_39_entropy_bytes);
    transfer_payload.set_tx_out_public_key(proto_tx_pubkey);
    transfer_payload.set_memo(memo);

    let mut wrapper = mc_mobilecoind_api::printable::PrintableWrapper::new();
    wrapper.set_transfer_payload(transfer_payload);

    Ok(wrapper.b58_encode()?)
}

pub fn b58_decode_transfer_payload(
    transfer_payload_b58: String,
) -> Result<DecodedTransferPayload, B58Error> {
    let wrapper = mc_mobilecoind_api::printable::PrintableWrapper::b58_decode(
        transfer_payload_b58.to_string(),
    )?;
    let transfer_payload = wrapper.get_transfer_payload();

    // Must have one type of entropy.
    if transfer_payload.get_root_entropy().is_empty()
        && transfer_payload.get_bip39_entropy().is_empty()
    {
        return Err(B58Error::InvalidEntropy);
    }

    // Only allow one type of entropy.
    if !transfer_payload.get_root_entropy().is_empty()
        && !transfer_payload.get_bip39_entropy().is_empty()
    {
        return Err(B58Error::InvalidEntropy);
    }

    // This will hold the account key.
    let mut account_key = None;

    // If we were provided with bip39 entropy, ensure it can be converted into a
    // mnemonic.
    let mut bip39_entropy = None;
    if !transfer_payload.get_bip39_entropy().is_empty() {
        match Mnemonic::from_entropy(transfer_payload.get_bip39_entropy(), Language::English) {
            Err(_) => {
                return Err(B58Error::InvalidEntropy);
            }
            Ok(mnemonic) => {
                bip39_entropy = Some(transfer_payload.get_bip39_entropy().to_vec());

                let key = mnemonic.derive_slip10_key(0);
                account_key = Some(AccountKey::from(key));
            }
        };
    }

    // If we were provided with root entropy, ensure it is 32 bytes long.
    let mut root_entropy = None;
    if !transfer_payload.get_root_entropy().is_empty() {
        if transfer_payload.get_root_entropy().len() != 32 {
            return Err(B58Error::InvalidEntropy);
        }

        let mut entropy = [0u8; 32];
        entropy.copy_from_slice(transfer_payload.get_root_entropy());
        root_entropy = Some(RootEntropy::from(&entropy));

        account_key = Some(AccountKey::from(&RootIdentity::from(&RootEntropy::from(
            &entropy,
        ))));
    }

    let txo_public_key =
        CompressedRistrettoPublic::try_from(transfer_payload.get_tx_out_public_key())?;

    Ok(DecodedTransferPayload {
        root_entropy,
        bip39_entropy,
        account_key: account_key.unwrap(), /* guaranteed to succeed because the code above either
                                            * manages to set it or returns an error. */
        txo_public_key,
        memo: transfer_payload.get_memo().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use crate::db::{b58_decode, b58_encode};
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
