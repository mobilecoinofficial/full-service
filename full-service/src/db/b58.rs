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
        mc_mobilecoind_api::printable::PrintableWrapper::b58_decode(b58_public_address.to_string())
            .unwrap();
    let pubaddr_proto: &mc_api::external::PublicAddress = if wrapper.has_payment_request() {
        let payment_request = wrapper.get_payment_request();
        payment_request.get_public_address()
    } else if wrapper.has_public_address() {
        wrapper.get_public_address()
    } else {
        return Err(WalletDbError::B58Decode);
    };
    Ok(PublicAddress::try_from(pubaddr_proto).unwrap())
}
