//! Public Address base58 encoding and decoding.

pub mod errors;
pub use self::errors::B58Error;

pub mod public_address_b58;

#[cfg(test)]
mod tests;

use bip39::{Language, Mnemonic};
use mc_account_keys::{AccountKey, PublicAddress, RootEntropy, RootIdentity};
use mc_api::printable::{
    printable_wrapper::Wrapper, PaymentRequest, PrintableWrapper, TransferPayload,
};
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
    match PrintableWrapper::b58_decode(b58_code)?.wrapper {
        Some(Wrapper::PublicAddress(_)) => Ok(PrintableWrapperType::PublicAddress),
        Some(Wrapper::PaymentRequest(_)) => Ok(PrintableWrapperType::PaymentRequest),
        Some(Wrapper::TransferPayload(_)) => Ok(PrintableWrapperType::TransferPayload),
        _ => Err(B58Error::NotPrintableWrapper),
    }
}

pub fn b58_encode_public_address(public_address: &PublicAddress) -> Result<String, B58Error> {
    let wrapper = PrintableWrapper {
        wrapper: Some(Wrapper::PublicAddress(public_address.into())),
    };
    Ok(wrapper.b58_encode()?)
}

pub fn b58_decode_public_address(public_address_b58_code: &str) -> Result<PublicAddress, B58Error> {
    let wrapper = PrintableWrapper::b58_decode(public_address_b58_code.to_string())?;

    let Some(Wrapper::PublicAddress(public_address_proto)) = wrapper.wrapper else {
        return Err(B58Error::NotPublicAddress);
    };

    Ok(PublicAddress::try_from(&public_address_proto)?)
}

pub fn b58_encode_payment_request(
    public_address: &PublicAddress,
    amount: &Amount,
    memo: String,
) -> Result<String, B58Error> {
    let payment_request = PaymentRequest {
        public_address: Some(public_address.into()),
        value: amount.value,
        token_id: *amount.token_id,
        memo,
        payment_id: 0,
    };

    let wrapper = PrintableWrapper {
        wrapper: Some(Wrapper::PaymentRequest(payment_request)),
    };

    Ok(wrapper.b58_encode()?)
}

pub fn b58_decode_payment_request(
    payment_request_b58: String,
) -> Result<DecodedPaymentRequest, B58Error> {
    let wrapper = PrintableWrapper::b58_decode(payment_request_b58)?;
    let Some(Wrapper::PaymentRequest(payment_request_message)) = wrapper.wrapper else {
        return Err(B58Error::NotPaymentRequest);
    };

    let public_address = PublicAddress::try_from(
        &payment_request_message
            .public_address
            .ok_or(B58Error::NotPaymentRequest)?,
    )?;
    let value = payment_request_message.value;
    let token_id = payment_request_message.token_id;
    let memo = payment_request_message.memo;

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
    let transfer_payload = TransferPayload {
        bip39_entropy: bip_39_entropy_bytes,
        tx_out_public_key: Some(proto_tx_pubkey),
        memo,
        ..Default::default()
    };

    let wrapper = PrintableWrapper {
        wrapper: Some(Wrapper::TransferPayload(transfer_payload)),
    };

    Ok(wrapper.b58_encode()?)
}

// Needed because root_entropy is deprecated
#[allow(deprecated)]
pub fn b58_decode_transfer_payload(
    transfer_payload_b58: String,
) -> Result<DecodedTransferPayload, B58Error> {
    let wrapper = PrintableWrapper::b58_decode(transfer_payload_b58)?;

    let Some(Wrapper::TransferPayload(transfer_payload)) = wrapper.wrapper else {
        return Err(B58Error::NotTransferPayload);
    };

    // Must have exactly one type of entropy.
    if transfer_payload.root_entropy.is_empty() && transfer_payload.bip39_entropy.is_empty() {
        return Err(B58Error::TransferPayloadRequiresSingleEntropy);
    }
    if !transfer_payload.root_entropy.is_empty() && !transfer_payload.bip39_entropy.is_empty() {
        return Err(B58Error::TransferPayloadRequiresSingleEntropy);
    }

    // This will hold the account key.
    let mut account_key = None;

    // If we were provided with bip39 entropy, ensure it can be converted into a
    // mnemonic.
    let mut bip39_entropy = None;
    if !transfer_payload.bip39_entropy.is_empty() {
        match Mnemonic::from_entropy(transfer_payload.bip39_entropy.as_slice(), Language::English) {
            Ok(mnemonic) => {
                bip39_entropy = Some(transfer_payload.bip39_entropy.clone());

                let key = mnemonic.derive_slip10_key(0);
                account_key = Some(AccountKey::from(key));
            }
            Err(_) => return Err(B58Error::InvalidEntropy),
        };
    }

    // If we were provided with root entropy, ensure it is 32 bytes long.
    let mut root_entropy = None;
    if !transfer_payload.root_entropy.is_empty() {
        if transfer_payload.root_entropy.len() != 32 {
            return Err(B58Error::InvalidEntropy);
        }

        let mut entropy = [0u8; 32];
        entropy.copy_from_slice(transfer_payload.root_entropy.as_slice());
        root_entropy = Some(RootEntropy::from(&entropy));

        account_key = Some(AccountKey::from(&RootIdentity::from(&RootEntropy::from(
            &entropy,
        ))));
    }

    let txo_public_key = CompressedRistrettoPublic::try_from(
        &transfer_payload
            .tx_out_public_key
            .ok_or(B58Error::NotTransferPayload)?,
    )?;

    Ok(DecodedTransferPayload {
        root_entropy,
        bip39_entropy,
        account_key: account_key.unwrap(), /* guaranteed to succeed because the code above either
                                            * manages to set it or returns an error. */
        txo_public_key,
        memo: transfer_payload.memo,
    })
}
