// Copyright (c) 2020-2021 MobileCoin Inc.

//! Service for managing transaction receipts.
//!
//! A transaction receipt is constructed at the same time a transaction is
//! constructed. It contains details about the outputs in the transaction, as
//! well as a confirmation proof for each output, linking the sender to the
//! output. The chooses whether to share this receipt with the recipient, for
//! example, in the case of a dispute.

use crate::{
    db::{models::Txo, WalletDbError},
    WalletService,
};
use displaydoc::Display;
use mc_account_keys::PublicAddress;
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_crypto_keys::CompressedRistrettoPublic;
use mc_fog_report_validation::FogPubkeyResolver;
use mc_transaction_core::tx::TxOutConfirmationNumber;
use std::convert::TryFrom;

/// Errors for the Address Service.
#[derive(Display, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum ReceiptServiceError {
    /// Error interacting with the database: {0}
    Database(WalletDbError),

    /// Diesel Error: {0}
    Diesel(diesel::result::Error),

    /// Error with LedgerDB: {0}
    LedgerDB(mc_ledger_db::Error),

    /// Error converting to/from API protos: {0}
    ProtoConversion(mc_api::ConversionError),

    /// Error Converting Proto but throws convert::Infallible.
    ProtoConversionInfallible,
}

impl From<WalletDbError> for ReceiptServiceError {
    fn from(src: WalletDbError) -> Self {
        Self::Database(src)
    }
}

impl From<diesel::result::Error> for ReceiptServiceError {
    fn from(src: diesel::result::Error) -> Self {
        Self::Diesel(src)
    }
}

impl From<mc_api::ConversionError> for ReceiptServiceError {
    fn from(src: mc_api::ConversionError) -> Self {
        Self::ProtoConversion(src)
    }
}

#[derive(Debug)]
pub struct ReceiverTxReceipt {
    /// The recipient of this Txo.
    recipient: PublicAddress,

    /// The public key of the Txo sent to the recipient.
    tx_public_key: CompressedRistrettoPublic,

    /// The hash of the Txo sent to the recipient.
    tx_out_hash: Vec<u8>,

    /// The tombstone block for the transaction.
    tombstone: u64,

    /// The proof for this Txo, which links the sender to this Txo.
    proof: TxOutConfirmationNumber,
}

impl TryFrom<&mc_mobilecoind_api::ReceiverTxReceipt> for ReceiverTxReceipt {
    type Error = ReceiptServiceError;

    fn try_from(
        src: &mc_mobilecoind_api::ReceiverTxReceipt,
    ) -> Result<ReceiverTxReceipt, ReceiptServiceError> {
        let recipient: PublicAddress = PublicAddress::try_from(src.get_recipient())?;
        let tx_public_key: CompressedRistrettoPublic =
            CompressedRistrettoPublic::try_from(src.get_tx_public_key())?;
        let mut proof_bytes = [0u8; 32];
        proof_bytes[0..32].copy_from_slice(src.get_confirmation_number());
        let proof = TxOutConfirmationNumber::from(&proof_bytes);
        Ok(ReceiverTxReceipt {
            recipient,
            tx_public_key,
            tx_out_hash: src.get_tx_out_hash().to_vec(),
            tombstone: src.get_tombstone(),
            proof,
        })
    }
}

/// Trait defining the ways in which the wallet can interact with and manage
/// receipts.
pub trait ReceiptService {
    /// Applies the transaction receipts from a sender to the wallet.
    ///
    /// Verifies the proof of each Txo, and updates the associated transaction
    /// logs.
    fn apply_receiver_receipts(
        &self,
        receiver_receipts: &[ReceiverTxReceipt],
    ) -> Result<Vec<Txo>, ReceiptServiceError>;
}

impl<T, FPR> ReceiptService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn apply_receiver_receipts(
        &self,
        receiver_receipts: &[ReceiverTxReceipt],
    ) -> Result<Vec<Txo>, ReceiptServiceError> {
        println!("{:?}", receiver_receipts);
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mc_account_keys::AccountKey;
    use mc_crypto_keys::RistrettoPrivate;
    use mc_crypto_rand::RngCore;
    use mc_transaction_core::tx::TxOut;
    use mc_util_from_random::FromRandom;
    use rand::{rngs::StdRng, SeedableRng};

    #[test]
    fn test_receipt_round_trip() {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let account_key = AccountKey::random(&mut rng);
        let public_address = account_key.default_subaddress();
        let txo = TxOut::new(
            rng.next_u64(),
            &public_address,
            &RistrettoPrivate::from_random(&mut rng),
            Default::default(),
        )
        .expect("Could not make TxOut");
        let tombstone = rng.next_u64();
        let mut proof_bytes = [0u8; 32];
        rng.fill_bytes(&mut proof_bytes);
        let confirmation_number = TxOutConfirmationNumber::from(proof_bytes);

        let mut proto_tx_receipt = mc_mobilecoind_api::ReceiverTxReceipt::new();
        proto_tx_receipt.set_recipient((&public_address).into());
        proto_tx_receipt.set_tx_public_key((&txo.public_key).into());
        proto_tx_receipt.set_tx_out_hash(txo.hash().to_vec());
        proto_tx_receipt.set_tombstone(tombstone);
        proto_tx_receipt.set_confirmation_number(confirmation_number.to_vec());

        let tx_receipt =
            ReceiverTxReceipt::try_from(&proto_tx_receipt).expect("Could not convert tx receipt");
        assert_eq!(public_address, tx_receipt.recipient);
        assert_eq!(txo.public_key, tx_receipt.tx_public_key);
        assert_eq!(txo.hash().to_vec(), tx_receipt.tx_out_hash);
        assert_eq!(tombstone, tx_receipt.tombstone);
        assert_eq!(confirmation_number, tx_receipt.proof);
    }
}
