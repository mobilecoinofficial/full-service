//! Public Address base58 encoding and decoding.

pub mod errors;
#[cfg(test)]
mod tests;
pub use self::errors::B58Error;

use bip39::{Language, Mnemonic};
use mc_account_keys::{AccountKey, PublicAddress, RootEntropy, RootIdentity};
use mc_api::printable::{PaymentRequest, PrintableWrapper, TransferPayload};
use mc_core::slip10::Slip10KeyGenerator;
use mc_crypto_keys::CompressedRistrettoPublic;
use mc_transaction_core::Amount;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

pub struct DecodedPaymentRequest {
    pub public_address: PublicAddress,
    pub value: u64,
    pub token_id: u64,
    pub memo: String,
}

pub struct DecodedTransferPayload {
    pub root_entropy: Option<RootEntropy>,
    pub bip39_entropy: Option<Vec<u8>>,
    pub account_key: AccountKey,
    pub txo_public_key: CompressedRistrettoPublic,
    pub memo: String,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum PrintableWrapperType {
    PublicAddress,
    PaymentRequest,
    TransferPayload,
}

pub fn b58_printable_wrapper_type(b58_code: String) -> Result<PrintableWrapperType, B58Error> {
    let wrapper = PrintableWrapper::b58_decode(b58_code)?;

    if wrapper.has_payment_request() {
        return Ok(PrintableWrapperType::PaymentRequest);
    } else if wrapper.has_transfer_payload() {
        return Ok(PrintableWrapperType::TransferPayload);
    } else if wrapper.has_public_address() {
        return Ok(PrintableWrapperType::PublicAddress);
    }

    Err(B58Error::NotPrintableWrapper)
}

pub fn b58_encode_public_address(public_address: &PublicAddress) -> Result<String, B58Error> {
    let mut wrapper = PrintableWrapper::new();
    wrapper.set_public_address(public_address.into());
    Ok(wrapper.b58_encode()?)
}

pub fn b58_decode_public_address(public_address_b58_code: &str) -> Result<PublicAddress, B58Error> {
    let wrapper = PrintableWrapper::b58_decode(public_address_b58_code.to_string())?;

    let public_address_proto = if wrapper.has_public_address() {
        wrapper.get_public_address()
    } else {
        return Err(B58Error::NotPublicAddress);
    };

    Ok(PublicAddress::try_from(public_address_proto)?)
}

pub fn b58_encode_payment_request(
    public_address: &PublicAddress,
    amount: &Amount,
    memo: String,
) -> Result<String, B58Error> {
    let mut payment_request = PaymentRequest::new();
    payment_request.set_public_address(public_address.into());
    payment_request.set_value(amount.value);
    payment_request.set_token_id(*amount.token_id);
    payment_request.set_memo(memo);

    let mut wrapper = PrintableWrapper::new();
    wrapper.set_payment_request(payment_request);

    Ok(wrapper.b58_encode()?)
}

pub fn b58_decode_payment_request(
    payment_request_b58: String,
) -> Result<DecodedPaymentRequest, B58Error> {
    let wrapper = PrintableWrapper::b58_decode(payment_request_b58)?;
    let payment_request_message = if wrapper.has_payment_request() {
        wrapper.get_payment_request()
    } else {
        return Err(B58Error::NotPaymentRequest);
    };

    let public_address = PublicAddress::try_from(payment_request_message.get_public_address())?;
    let value = payment_request_message.get_value();
    let token_id = payment_request_message.get_token_id();
    let memo = payment_request_message.get_memo().to_string();

    Ok(DecodedPaymentRequest {
        public_address,
        value,
        token_id,
        memo,
    })
}

pub fn b58_encode_transfer_payload(
    bip_39_entropy_bytes: Vec<u8>,
    proto_tx_pubkey: mc_api::external::CompressedRistretto,
    memo: String,
) -> Result<String, B58Error> {
    let mut transfer_payload = TransferPayload::new();
    transfer_payload.set_bip39_entropy(bip_39_entropy_bytes);
    transfer_payload.set_tx_out_public_key(proto_tx_pubkey);
    transfer_payload.set_memo(memo);

    let mut wrapper = PrintableWrapper::new();
    wrapper.set_transfer_payload(transfer_payload);

    Ok(wrapper.b58_encode()?)
}

pub fn b58_decode_transfer_payload(
    transfer_payload_b58: String,
) -> Result<DecodedTransferPayload, B58Error> {
    let wrapper = PrintableWrapper::b58_decode(transfer_payload_b58)?;

    let transfer_payload = if wrapper.has_transfer_payload() {
        wrapper.get_transfer_payload()
    } else {
        return Err(B58Error::NotTransferPayload);
    };

    // Must have exactly one type of entropy.
    if transfer_payload.get_root_entropy().is_empty()
        && transfer_payload.get_bip39_entropy().is_empty()
    {
        return Err(B58Error::TransferPayloadRequiresSingleEntropy);
    }
    if !transfer_payload.get_root_entropy().is_empty()
        && !transfer_payload.get_bip39_entropy().is_empty()
    {
        return Err(B58Error::TransferPayloadRequiresSingleEntropy);
    }

    // This will hold the account key.
    let mut account_key = None;

    // If we were provided with bip39 entropy, ensure it can be converted into a
    // mnemonic.
    let mut bip39_entropy = None;
    if !transfer_payload.get_bip39_entropy().is_empty() {
        match Mnemonic::from_entropy(transfer_payload.get_bip39_entropy(), Language::English) {
            Ok(mnemonic) => {
                bip39_entropy = Some(transfer_payload.get_bip39_entropy().to_vec());

                let key = mnemonic.derive_slip10_key(0);
                account_key = Some(AccountKey::from(key));
            }
            Err(_) => return Err(B58Error::InvalidEntropy),
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
