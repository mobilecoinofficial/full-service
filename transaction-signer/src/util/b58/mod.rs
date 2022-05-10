use mc_account_keys::PublicAddress;
use mc_api::printable::PrintableWrapper;
use std::convert::TryFrom;

pub fn b58_decode_public_address(public_address_b58_code: &str) -> PublicAddress {
    let wrapper = PrintableWrapper::b58_decode(public_address_b58_code.to_string()).unwrap();
    PublicAddress::try_from(wrapper.get_public_address()).unwrap()
}

pub fn b58_encode_public_address(public_address: &PublicAddress) -> String {
    let mut wrapper = PrintableWrapper::new();
    wrapper.set_public_address(public_address.into());
    wrapper.b58_encode().unwrap()
}
