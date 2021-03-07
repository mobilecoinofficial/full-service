// Copyright (c) 2020-2021 MobileCoin Inc.

//! Service for managing transaction receipts.
//!
//! A transaction receipt is constructed at the same time a transaction is
//! constructed. It contains details about the outputs in the transaction, as
//! well as a confirmation proof for each output, linking the sender to the
//! output. The chooses whether to share this receipt with the recipient, for
//! example, in the case of a dispute.

use crate::{
    db::{
        models::{AccountTxoStatus, Txo, TXO_STATUS_SECRETED, TXO_TYPE_MINTED},
        txo::TxoModel,
        WalletDbError,
    },
    WalletService,
};
use displaydoc::Display;
use mc_account_keys::PublicAddress;
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_crypto_keys::CompressedRistrettoPublic;
use mc_fog_report_validation::FogPubkeyResolver;
use mc_mobilecoind::payments::TxProposal;
use mc_transaction_core::tx::TxOutConfirmationNumber;
use serde::{Deserialize, Serialize};
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
    txo_public_key: CompressedRistrettoPublic,

    /// The hash of the Txo sent to the recipient.
    txo_hash: Vec<u8>,

    /// The tombstone block for the transaction.
    tombstone: u64,

    /// The proof for this Txo, which links the sender to this Txo.
    proof: TxOutConfirmationNumber,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ReceiptTransactionStatus {
    /// All Txos are in the ledger at the same block index, and the expected
    /// value matches the value of the Txos.
    TransactionSuccess,

    /// No Txos have landed in the wallet yet.
    TransactionPending,

    /// All Txos are in the ledger, at different block indices. This indicates
    /// the Txos were spent in different transactions, and the receipt is
    /// invalid.
    TxosReceivedAtDifferentBlockIndices,

    /// Some Txos received, some not. This indicates the Txos were spent in
    /// different transactions, and the receipt is invalid.
    SomeTxosMissing,

    /// The expected value of the Txos did not match the actual value.
    UnexpectedValue,
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
            txo_public_key: tx_public_key,
            txo_hash: src.get_tx_out_hash().to_vec(),
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
        sender_metadata: String,
    ) -> Result<Vec<Txo>, ReceiptServiceError>;

    /// Check status
    fn check_receiver_receipts_status(
        &self,
        receiver_receipts: &[ReceiverTxReceipt],
        expected_value: u64,
    ) -> Result<ReceiptTransactionStatus, ReceiptServiceError>;

    /// Create a receipt from a given TxProposal
    fn create_receiver_receipts(
        &self,
        tx_proposal: &TxProposal,
    ) -> Result<Vec<ReceiverTxReceipt>, ReceiptServiceError>;
}

impl<T, FPR> ReceiptService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn apply_receiver_receipts(
        &self,
        receiver_receipts: &[ReceiverTxReceipt],
        sender_metadata: String,
    ) -> Result<Vec<Txo>, ReceiptServiceError> {
        //let txos = Txo::select_by_public_key()

        Ok(vec![])
    }

    fn check_receiver_receipts_status(
        &self,
        receiver_receipts: &[ReceiverTxReceipt],
        expected_value: u64,
    ) -> Result<ReceiptTransactionStatus, ReceiptServiceError> {
        let public_keys: Vec<&CompressedRistrettoPublic> = receiver_receipts
            .iter()
            .map(|r| &r.txo_public_key)
            .collect();
        let txos_and_statuses =
            Txo::select_by_public_key(&public_keys, &self.wallet_db.get_conn()?)?;
        // None of the Txos from the receipts are in this wallet.
        if txos_and_statuses.len() == 0 {
            return Ok(ReceiptTransactionStatus::TransactionPending);
        }

        // Figure out which Txos were minted by us and have not yet been received by us.
        // (For to-self transactions).
        let minted: Vec<&(Txo, AccountTxoStatus)> = txos_and_statuses
            .iter()
            .filter(|(txo, status)| {
                status.txo_type == TXO_TYPE_MINTED && status.txo_status == TXO_STATUS_SECRETED
            })
            .collect();

        // Need to verify if the Txos in the wallet were minted by us - in which
        // case, this transaction could be pending.
        if minted.len() == receiver_receipts.len() {
            return Ok(ReceiptTransactionStatus::TransactionPending);
        }

        // Some of the Txos are in this wallet, but some are missing. The receipt is
        // malformed.
        if txos_and_statuses.len() != receiver_receipts.len() {
            // FIXME also minted?
            return Ok(ReceiptTransactionStatus::SomeTxosMissing);
        }

        // We have received all the Txos in this wallet. Check that they're in the same
        // block index.
        let received_block_indices: Vec<u64> = txos_and_statuses
            .iter()
            .map(|(txo, status)| txo.received_block_index)
            .filter_map(|b| b.map(|i| i as u64))
            .collect();
        if received_block_indices.iter().min() != received_block_indices.iter().max() {
            return Ok(ReceiptTransactionStatus::TxosReceivedAtDifferentBlockIndices);
        }

        // Check that the value of the received Txos matches the expected value
        let received_total: u64 = txos_and_statuses
            .iter()
            .map(|(txo, status)| txo.value as u64)
            .collect::<Vec<u64>>()
            .iter()
            .sum();
        if received_total != expected_value {
            return Ok(ReceiptTransactionStatus::UnexpectedValue);
        }

        Ok(ReceiptTransactionStatus::TransactionSuccess)
    }

    fn create_receiver_receipts(
        &self,
        tx_proposal: &TxProposal,
    ) -> Result<Vec<ReceiverTxReceipt>, ReceiptServiceError> {
        let receiver_tx_receipts: Vec<ReceiverTxReceipt> = tx_proposal
            .outlays
            .iter()
            .enumerate()
            .map(|(outlay_index, outlay)| {
                let tx_out_index = tx_proposal.outlay_index_to_tx_out_index[&outlay_index];
                let tx_out = tx_proposal.tx.prefix.outputs[tx_out_index].clone();
                ReceiverTxReceipt {
                    recipient: outlay.clone().receiver,
                    txo_public_key: tx_out.public_key,
                    txo_hash: tx_out.hash().to_vec(),
                    tombstone: tx_proposal.tx.prefix.tombstone_block,
                    proof: tx_proposal.outlay_confirmation_numbers[outlay_index].clone(),
                }
            })
            .collect::<Vec<ReceiverTxReceipt>>();
        Ok(receiver_tx_receipts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db::{
            account::AccountID,
            b58_decode,
            models::{TransactionLog, TX_DIRECTION_SENT},
            transaction_log::{AssociatedTxos, TransactionLogModel},
        },
        service::{
            account::AccountService, address::AddressService, proof::ProofService,
            transaction::TransactionService, transaction_log::TransactionLogService,
            txo::TxoService,
        },
        test_utils::{
            add_block_to_ledger_db, add_block_with_tx_proposal, get_test_ledger,
            manually_sync_account, setup_wallet_service, MOB,
        },
    };
    use mc_account_keys::AccountKey;
    use mc_common::logger::{test_with_logger, Logger};
    use mc_crypto_keys::RistrettoPrivate;
    use mc_crypto_rand::RngCore;
    use mc_transaction_core::{ring_signature::KeyImage, tx::TxOut};
    use mc_util_from_random::FromRandom;
    use rand::{rngs::StdRng, SeedableRng};

    // The receipt should convert between the rust and proto representations.
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
        assert_eq!(txo.public_key, tx_receipt.txo_public_key);
        assert_eq!(txo.hash().to_vec(), tx_receipt.txo_hash);
        assert_eq!(tombstone, tx_receipt.tombstone);
        assert_eq!(confirmation_number, tx_receipt.proof);
    }

    #[test_with_logger]
    fn test_create_receipt(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        let service = setup_wallet_service(ledger_db.clone(), logger.clone());
        let alice = service
            .create_account(Some("Alice's Main Account".to_string()), None)
            .unwrap();

        // Fund Alice
        let alice_account_key: AccountKey = mc_util_serial::decode(&alice.account_key).unwrap();
        let alice_public_address = alice_account_key.subaddress(alice.main_subaddress_index as u64);
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![alice_public_address.clone()],
            100 * MOB as u64,
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );
        manually_sync_account(
            &ledger_db,
            &service.wallet_db,
            &AccountID(alice.account_id_hex.to_string()),
            13,
            &logger,
        );

        let bob = service
            .create_account(Some("Bob's Main Account".to_string()), None)
            .unwrap();
        let bob_addresses = service
            .get_all_addresses_for_account(&AccountID(bob.account_id_hex.clone()))
            .expect("Could not get addresses for Bob");
        let bob_address = bob_addresses[0].assigned_subaddress_b58.clone();

        // Create a TxProposal to Bob
        let tx_proposal = service
            .build_transaction(
                &alice.account_id_hex,
                &bob_address,
                (24 * MOB).to_string(),
                None,
                None,
                None,
                None,
            )
            .expect("Could not build transaction");

        let receipts = service
            .create_receiver_receipts(&tx_proposal)
            .expect("Could not create receiver receipts");

        // Note: Since we manually added the block rather than using "Submit," we need
        // to manually log submitted. This needs to happen before it hits the ledger, or
        // else we will get a Unique constraint failed if we had already scanned
        // before logging submitted.
        TransactionLog::log_submitted(
            tx_proposal.clone(),
            14,
            "".to_string(),
            Some(&alice.account_id_hex),
            &service.wallet_db.get_conn().unwrap(),
        )
        .expect("Could not log submitted");

        // Add the txo to the ledger
        add_block_with_tx_proposal(&mut ledger_db, tx_proposal);
        manually_sync_account(
            &ledger_db,
            &service.wallet_db,
            &AccountID(alice.account_id_hex.to_string()),
            14,
            &logger,
        );
        manually_sync_account(
            &ledger_db,
            &service.wallet_db,
            &AccountID(bob.account_id_hex.to_string()),
            14,
            &logger,
        );

        // Get corresponding Txo for Bob
        let txos = service
            .list_txos(&AccountID(bob.account_id_hex))
            .expect("Could not get Bob Txos");
        assert_eq!(txos.len(), 1);

        // Get the corresponding TransactionLog for Alice's Account - only the sender
        // has the proof.
        let transaction_logs = service
            .list_transaction_logs(&AccountID(alice.account_id_hex))
            .expect("Could not get transaction logs");
        // Alice should have two received (initial and change), and one sent
        // TransactionLog.
        assert_eq!(transaction_logs.len(), 3);
        let sent_transaction_logs_and_associated_txos: Vec<&(TransactionLog, AssociatedTxos)> =
            transaction_logs
                .iter()
                .filter(|t| t.0.direction == TX_DIRECTION_SENT)
                .collect();
        assert_eq!(sent_transaction_logs_and_associated_txos.len(), 1);
        let sent_transaction_log: TransactionLog =
            sent_transaction_logs_and_associated_txos[0].0.clone();
        let proofs = service
            .get_proofs(&sent_transaction_log.transaction_id_hex)
            .expect("Could not get proofs");
        assert_eq!(proofs.len(), 1);

        assert_eq!(receipts.len(), 1);
        assert_eq!(
            receipts[0].recipient,
            b58_decode(&bob_address).expect("Could not decode public address")
        );
        let txo_pubkey =
            mc_util_serial::decode(&txos[0].txo.public_key).expect("Could not decode pubkey");
        assert_eq!(receipts[0].txo_public_key, txo_pubkey);
        assert_eq!(receipts[0].tombstone, 63); // Ledger seeded with 12 blocks at tx construction, then one appended + 50
        let txo: TxOut = mc_util_serial::decode(&txos[0].txo.txo).expect("Could not decode txo");
        assert_eq!(receipts[0].txo_hash, txo.hash());
        assert_eq!(receipts[0].proof, proofs[0].proof);
    }

    // All txos received should return TransactionSuccess
    #[test_with_logger]
    fn test_check_receiver_receipts_status_success(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        let service = setup_wallet_service(ledger_db.clone(), logger);
        let alice = service
            .create_account(Some("Alice's Main Account".to_string()), None)
            .unwrap();

        // Fund Alice
        let alice_account_key: AccountKey = mc_util_serial::decode(&alice.account_key).unwrap();
        let alice_public_address = alice_account_key.subaddress(alice.main_subaddress_index as u64);
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![alice_public_address.clone()],
            100 * MOB as u64,
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        let bob = service
            .create_account(Some("Bob's Main Account".to_string()), None)
            .unwrap();

        // Alice sends a transaction to Bob
    }
}
