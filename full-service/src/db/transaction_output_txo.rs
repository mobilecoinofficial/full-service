use super::{models::TransactionOutputTxo, WalletDbError};
use crate::util::b58::b58_decode_public_address;
use mc_account_keys::PublicAddress;

impl TransactionOutputTxo {
    pub fn recipient_public_address(&self) -> Result<PublicAddress, WalletDbError> {
        Ok(b58_decode_public_address(
            &self.recipient_public_address_b58,
        )?)
    }
}
