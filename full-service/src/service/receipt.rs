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
        account::AccountID,
        models::{Txo, TXO_STATUS_SECRETED, TXO_TYPE_MINTED},
        txo::{TxoID, TxoModel},
        WalletDbError,
    },
    service::proof::{ProofService, ProofServiceError},
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

    /// Error with the Proof Service
    ProofService(ProofServiceError),
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

impl From<ProofServiceError> for ReceiptServiceError {
    fn from(src: ProofServiceError) -> Self {
        Self::ProofService(src)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ReceiverReceipt {
    /// The recipient of this Txo.
    pub recipient: PublicAddress,

    /// The public key of the Txo sent to the recipient.
    pub txo_public_key: CompressedRistrettoPublic,

    /// The hash of the Txo sent to the recipient.
    pub txo_hash: Vec<u8>,

    /// The tombstone block for the transaction.
    pub tombstone: u64,

    /// The proof for this Txo, which links the sender to this Txo.
    pub proof: TxOutConfirmationNumber,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
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

    /// The expected value of the Txos did not match the actual value.
    UnexpectedValue,

    /// Invalid proof
    InvalidProof,

    /// Receipt contains duplicate Txos
    DuplicateTxos,
}

impl TryFrom<&mc_mobilecoind_api::ReceiverTxReceipt> for ReceiverReceipt {
    type Error = ReceiptServiceError;

    fn try_from(
        src: &mc_mobilecoind_api::ReceiverTxReceipt,
    ) -> Result<ReceiverReceipt, ReceiptServiceError> {
        let recipient: PublicAddress = PublicAddress::try_from(src.get_recipient())?;
        let tx_public_key: CompressedRistrettoPublic =
            CompressedRistrettoPublic::try_from(src.get_tx_public_key())?;
        let mut proof_bytes = [0u8; 32];
        proof_bytes[0..32].copy_from_slice(src.get_confirmation_number());
        let proof = TxOutConfirmationNumber::from(&proof_bytes);
        Ok(ReceiverReceipt {
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
    /// Check the status of the Txos in the receipts.
    ///
    /// Applies the proofs by verifying the proofs once the Txos have landed.
    fn check_receiver_receipt_status(
        &self,
        account_id: &AccountID,
        receiver_receipt: &ReceiverReceipt,
        expected_value: u64,
    ) -> Result<ReceiptTransactionStatus, ReceiptServiceError>;

    /// Create a receipt from a given TxProposal
    fn create_receiver_receipts(
        &self,
        tx_proposal: &TxProposal,
    ) -> Result<Vec<ReceiverReceipt>, ReceiptServiceError>;
}

impl<T, FPR> ReceiptService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn check_receiver_receipt_status(
        &self,
        account_id: &AccountID,
        receiver_receipt: &ReceiverReceipt,
        expected_value: u64,
    ) -> Result<ReceiptTransactionStatus, ReceiptServiceError> {
        // Get the transaction from the database, with status.
        let txos_and_statuses = Txo::select_by_public_key(
            account_id,
            &[&receiver_receipt.txo_public_key],
            &self.wallet_db.get_conn()?,
        )?;

        // Return if the Txo from the receipt is not in this wallet yet.
        if txos_and_statuses.is_empty() {
            return Ok(ReceiptTransactionStatus::TransactionPending);
        }
        let (txo, status) = &txos_and_statuses[0];

        // Figure out whether the Txo was minted by us, and has not yet been received by
        // us. (For to-self transactions). If the Txo was minted by us, this
        // transaction is pending.
        if status.txo_type == TXO_TYPE_MINTED && status.txo_status == TXO_STATUS_SECRETED {
            return Ok(ReceiptTransactionStatus::TransactionPending);
        }

        // Check that the value of the received Txo matches the expected value.
        if (txo.value as u64) != expected_value {
            return Ok(ReceiptTransactionStatus::UnexpectedValue);
        }

        // Verify the proof.
        let proof_hex = hex::encode(mc_util_serial::encode(&receiver_receipt.proof));
        if !self.verify_proof(account_id, &TxoID(txo.txo_id_hex.clone()), &proof_hex)? {
            return Ok(ReceiptTransactionStatus::InvalidProof);
        }

        Ok(ReceiptTransactionStatus::TransactionSuccess)
    }

    fn create_receiver_receipts(
        &self,
        tx_proposal: &TxProposal,
    ) -> Result<Vec<ReceiverReceipt>, ReceiptServiceError> {
        let receiver_tx_receipts: Vec<ReceiverReceipt> = tx_proposal
            .outlays
            .iter()
            .enumerate()
            .map(|(outlay_index, outlay)| {
                let tx_out_index = tx_proposal.outlay_index_to_tx_out_index[&outlay_index];
                let tx_out = tx_proposal.tx.prefix.outputs[tx_out_index].clone();
                ReceiverReceipt {
                    recipient: outlay.clone().receiver,
                    txo_public_key: tx_out.public_key,
                    txo_hash: tx_out.hash().to_vec(),
                    tombstone: tx_proposal.tx.prefix.tombstone_block,
                    proof: tx_proposal.outlay_confirmation_numbers[outlay_index].clone(),
                }
            })
            .collect::<Vec<ReceiverReceipt>>();
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
            account::AccountService, address::AddressService, transaction::TransactionService,
            transaction_log::TransactionLogService, txo::TxoService,
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
            ReceiverReceipt::try_from(&proto_tx_receipt).expect("Could not convert tx receipt");
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
        let receipt = &receipts[0];

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

        assert_eq!(
            receipt.recipient,
            b58_decode(&bob_address).expect("Could not decode public address")
        );
        let txo_pubkey =
            mc_util_serial::decode(&txos[0].txo.public_key).expect("Could not decode pubkey");
        assert_eq!(receipt.txo_public_key, txo_pubkey);
        assert_eq!(receipt.tombstone, 63); // Ledger seeded with 12 blocks at tx construction, then one appended + 50
        let txo: TxOut = mc_util_serial::decode(&txos[0].txo.txo).expect("Could not decode txo");
        assert_eq!(receipt.txo_hash, txo.hash());
        assert_eq!(receipt.proof, proofs[0].proof);
    }

    // All txos received should return TransactionSuccess, and TransactionPending
    // until they are received.
    #[test_with_logger]
    fn test_check_receiver_receipt_status_success(logger: Logger) {
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
        let bob_account_id = AccountID(bob.account_id_hex.to_string());

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
        let receipt = &receipts[0];

        // Bob checks the status of the receipts.
        let status = service
            .check_receiver_receipt_status(&bob_account_id, &receipt, 24 * MOB as u64)
            .expect("Could not check status of receipt");

        // Status should be pending until block lands and is scanned
        assert_eq!(status, ReceiptTransactionStatus::TransactionPending);

        // Land the Txo in the ledger - only sync for the sender
        TransactionLog::log_submitted(
            tx_proposal.clone(),
            14,
            "".to_string(),
            Some(&alice.account_id_hex),
            &service.wallet_db.get_conn().unwrap(),
        )
        .expect("Could not log submitted");

        // Status for Bob should still be pending, even though the Txos will show up in
        // the wallet, but under Alice's account.
        let status = service
            .check_receiver_receipt_status(&bob_account_id, &receipt, 24 * MOB as u64)
            .expect("Could not check status of receipt");
        assert_eq!(status, ReceiptTransactionStatus::TransactionPending);

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

        // Status for Bob is succeeded.
        let status = service
            .check_receiver_receipt_status(&bob_account_id, &receipt, 24 * MOB as u64)
            .expect("Could not check status of receipt");
        assert_eq!(status, ReceiptTransactionStatus::TransactionSuccess);

        // Status for Alice would be pending, because she never received (and never will
        // receive) the Txos.
        let status = service
            .check_receiver_receipt_status(
                &AccountID(alice.account_id_hex),
                &receipt,
                24 * MOB as u64,
            )
            .expect("Could not check status of receipt");
        assert_eq!(status, ReceiptTransactionStatus::TransactionPending);
    }

    #[test_with_logger]
    fn test_check_receiver_receipt_status_wrong_value(logger: Logger) {
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
        let bob_account_id = AccountID(bob.account_id_hex.to_string());

        // Create a TxProposal to Bob
        let tx_proposal0 = service
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
            .create_receiver_receipts(&tx_proposal0)
            .expect("Could not create receiver receipt");
        let receipt0 = &receipts[0];

        // Land the Txo in the ledger - only sync for the sender
        TransactionLog::log_submitted(
            tx_proposal0.clone(),
            14,
            "".to_string(),
            Some(&alice.account_id_hex),
            &service.wallet_db.get_conn().unwrap(),
        )
        .expect("Could not log submitted");
        add_block_with_tx_proposal(&mut ledger_db, tx_proposal0);
        manually_sync_account(
            &ledger_db,
            &service.wallet_db,
            &AccountID(alice.account_id_hex.to_string()),
            14,
            &logger,
        );
        manually_sync_account(&ledger_db, &service.wallet_db, &bob_account_id, 14, &logger);

        // Bob checks the status, and is expecting an incorrect value
        let status = service
            .check_receiver_receipt_status(&bob_account_id, &receipt0, 18 * MOB as u64)
            .expect("Could not check status of receipt");
        assert_eq!(status, ReceiptTransactionStatus::UnexpectedValue);

        // Status for Alice would be pending, because she never received (and never will
        // receive) the Txos.
        let status = service
            .check_receiver_receipt_status(
                &AccountID(alice.account_id_hex),
                &receipt0,
                18 * MOB as u64,
            )
            .expect("Could not check status of receipt");
        assert_eq!(status, ReceiptTransactionStatus::TransactionPending);
    }

    #[test_with_logger]
    fn test_check_receiver_receipt_status_invalid_proof(logger: Logger) {
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
        let bob_account_id = AccountID(bob.account_id_hex.to_string());

        // Create a TxProposal to Bob
        let tx_proposal0 = service
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
            .create_receiver_receipts(&tx_proposal0)
            .expect("Could not create receiver receipts");
        let receipt0 = &receipts[0];

        // Land the Txo in the ledger - only sync for the sender
        TransactionLog::log_submitted(
            tx_proposal0.clone(),
            14,
            "".to_string(),
            Some(&alice.account_id_hex),
            &service.wallet_db.get_conn().unwrap(),
        )
        .expect("Could not log submitted");
        add_block_with_tx_proposal(&mut ledger_db, tx_proposal0);
        manually_sync_account(
            &ledger_db,
            &service.wallet_db,
            &AccountID(alice.account_id_hex.to_string()),
            14,
            &logger,
        );
        manually_sync_account(&ledger_db, &service.wallet_db, &bob_account_id, 14, &logger);

        // Construct an invalid receipt with an incorrect proof
        let mut receipt = receipt0.clone();
        let mut bad_proof_bytes = [0u8; 32];
        rng.fill_bytes(&mut bad_proof_bytes);
        let bad_proof = TxOutConfirmationNumber::from(bad_proof_bytes);
        receipt.proof = bad_proof;

        // Bob checks the status, and is expecting an incorrect value
        let status = service
            .check_receiver_receipt_status(&bob_account_id, &receipt, 24 * MOB as u64)
            .expect("Could not check status of receipt");
        assert_eq!(status, ReceiptTransactionStatus::InvalidProof);

        // Checking for the sender will be pending because the Txos haven't landed for
        // alice (and never will).
        let status = service
            .check_receiver_receipt_status(
                &AccountID(alice.account_id_hex),
                &receipt,
                18 * MOB as u64,
            )
            .expect("Could not check status of receipt");
        assert_eq!(status, ReceiptTransactionStatus::TransactionPending);
    }
}
