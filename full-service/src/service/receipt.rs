// Copyright (c) 2020-2021 MobileCoin Inc.

//! Service for managing transaction receipts.
//!
//! A transaction receipt is constructed at the same time a transaction is
//! constructed. It contains details about the outputs in the transaction, as
//! well as a confirmation number for each output, linking the sender to the
//! output. The chooses whether to share this receipt with the recipient, for
//! example, in the case of a dispute.

use crate::{
    db::{
        account::{AccountID, AccountModel},
        assigned_subaddress::AssignedSubaddressModel,
        models::{Account, AssignedSubaddress, Txo},
        txo::TxoModel,
        WalletDbError,
    },
    WalletService,
};
use diesel::Connection;
use displaydoc::Display;
use mc_account_keys::AccountKey;
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_crypto_keys::{CompressedRistrettoPublic, RistrettoPublic};
use mc_fog_report_validation::FogPubkeyResolver;
use mc_mobilecoind::payments::TxProposal;
use mc_transaction_core::{
    get_tx_out_shared_secret, tx::TxOutConfirmationNumber, Amount, AmountError,
};
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

    /// Error decoding prost: {0}
    ProstDecode(mc_util_serial::DecodeError),

    /// Error with crypto keys: {0}
    CryptoKey(mc_crypto_keys::KeyError),

    /// Error decoding from hex: {0}
    HexDecode(hex::FromHexError),
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

impl From<mc_util_serial::DecodeError> for ReceiptServiceError {
    fn from(src: mc_util_serial::DecodeError) -> Self {
        Self::ProstDecode(src)
    }
}

impl From<mc_crypto_keys::KeyError> for ReceiptServiceError {
    fn from(src: mc_crypto_keys::KeyError) -> Self {
        Self::CryptoKey(src)
    }
}

impl From<hex::FromHexError> for ReceiptServiceError {
    fn from(src: hex::FromHexError) -> Self {
        Self::HexDecode(src)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ReceiverReceipt {
    /// The public key of the Txo sent to the recipient.
    pub public_key: CompressedRistrettoPublic,

    /// The confirmation number for this Txo, which links the sender to this
    /// Txo.
    pub confirmation: TxOutConfirmationNumber,

    /// The tombstone block for the transaction.
    pub tombstone_block: u64,

    /// The encrypted amount of this transaction.
    /// Note: This value is self-reported by the sender and is unverifiable.
    pub amount: Amount,
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

    /// Invalid confirmation number.
    InvalidConfirmation,

    /// Receipt contains duplicate Txos
    DuplicateTxos,

    /// Receipt Amount does not match the Amount in the Txo: {0}
    AmountMismatch(String),

    /// Failed to decrypt the amount for the given Txo
    FailedAmountDecryption,
}

impl TryFrom<&mc_api::external::Receipt> for ReceiverReceipt {
    type Error = ReceiptServiceError;

    fn try_from(src: &mc_api::external::Receipt) -> Result<ReceiverReceipt, ReceiptServiceError> {
        let public_key: CompressedRistrettoPublic =
            CompressedRistrettoPublic::try_from(src.get_public_key())?;
        let confirmation = TxOutConfirmationNumber::try_from(src.get_confirmation())?;
        let amount = Amount::try_from(src.get_amount())?;
        Ok(ReceiverReceipt {
            public_key,
            confirmation,
            tombstone_block: src.get_tombstone_block(),
            amount,
        })
    }
}

/// Trait defining the ways in which the wallet can interact with and manage
/// receipts.
pub trait ReceiptService {
    /// Check the status of the Txos in the receipts.
    ///
    /// Validates confirmation numbers once the Txos have landed.
    fn check_receipt_status(
        &self,
        address: &str,
        receiver_receipt: &ReceiverReceipt,
    ) -> Result<(ReceiptTransactionStatus, Option<Txo>), ReceiptServiceError>;

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
    fn check_receipt_status(
        &self,
        address: &str,
        receiver_receipt: &ReceiverReceipt,
    ) -> Result<(ReceiptTransactionStatus, Option<Txo>), ReceiptServiceError> {
        let conn = &self.wallet_db.get_conn()?;
        conn.transaction(|| {
            let assigned_address = AssignedSubaddress::get(address, conn)?;
            let account_id = AccountID(assigned_address.account_id_hex.clone());
            let account = Account::get(&account_id, conn)?;
            // Get the transaction from the database, with status.
            let txos = Txo::select_by_public_key(&[&receiver_receipt.public_key], conn)?;

            // Return if the Txo from the receipt is not in this wallet yet.
            if txos.is_empty() {
                return Ok((ReceiptTransactionStatus::TransactionPending, None));
            }
            let txo = txos[0].clone();

            // Return if the Txo from the receipt has a pending tombstone block index
            if txo.pending_tombstone_block_index.is_some() {
                return Ok((ReceiptTransactionStatus::TransactionPending, Some(txo)));
            }

            // We are reproducing a bug with this logic and it is not how the
            // feature is intended to work. Currently, this will return a Transaction
            // Pending result even if the txo hits the ledger but does not belong to
            // this account. It should instead fall through to the next block of code
            // and return a FailedAmountDecryption, since that is more truly a description
            // of what happened.
            // TODO - remove this and it will work as expected
            if txo.minted_account_id_hex == Some(assigned_address.account_id_hex.clone())
                && txo.received_account_id_hex != Some(assigned_address.account_id_hex)
            {
                return Ok((ReceiptTransactionStatus::TransactionPending, Some(txo)));
            }

            // Decrypt the amount to get the expected value
            let account_key: AccountKey = mc_util_serial::decode(&account.account_key)?;
            let public_key: RistrettoPublic =
                RistrettoPublic::try_from(&receiver_receipt.public_key)?;
            let shared_secret =
                get_tx_out_shared_secret(account_key.view_private_key(), &public_key);
            let expected_value = match receiver_receipt.amount.get_value(&shared_secret) {
                Ok((v, _blinding)) => v,
                Err(AmountError::InconsistentCommitment) => {
                    return Ok((ReceiptTransactionStatus::FailedAmountDecryption, Some(txo)))
                }
            };
            // Check that the value of the received Txo matches the expected value.
            if (txo.value as u64) != expected_value {
                return Ok((
                    ReceiptTransactionStatus::AmountMismatch(format!(
                        "Expected: {}, Got: {}",
                        expected_value, txo.value
                    )),
                    Some(txo),
                ));
            }

            // Validate the confirmation number.
            let confirmation_hex =
                hex::encode(mc_util_serial::encode(&receiver_receipt.confirmation));
            let confirmation: TxOutConfirmationNumber =
                mc_util_serial::decode(&hex::decode(confirmation_hex)?)?;
            if !Txo::validate_confirmation(&account_id, &txo.txo_id_hex, &confirmation, conn)? {
                return Ok((ReceiptTransactionStatus::InvalidConfirmation, Some(txo)));
            }

            Ok((ReceiptTransactionStatus::TransactionSuccess, Some(txo)))
        })
    }

    fn create_receiver_receipts(
        &self,
        tx_proposal: &TxProposal,
    ) -> Result<Vec<ReceiverReceipt>, ReceiptServiceError> {
        let receiver_tx_receipts: Vec<ReceiverReceipt> = tx_proposal
            .outlays
            .iter()
            .enumerate()
            .map(|(outlay_index, _outlay)| {
                let tx_out_index = tx_proposal.outlay_index_to_tx_out_index[&outlay_index];
                let tx_out = tx_proposal.tx.prefix.outputs[tx_out_index].clone();
                ReceiverReceipt {
                    public_key: tx_out.public_key,
                    tombstone_block: tx_proposal.tx.prefix.tombstone_block,
                    confirmation: tx_proposal.outlay_confirmation_numbers[outlay_index].clone(),
                    amount: tx_out.amount,
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
            models::{TransactionLog, TX_DIRECTION_SENT},
            transaction_log::{AssociatedTxos, TransactionLogModel},
        },
        service::{
            account::AccountService, address::AddressService,
            confirmation_number::ConfirmationService, transaction::TransactionService,
            transaction_log::TransactionLogService, txo::TxoService,
        },
        test_utils::{
            add_block_to_ledger_db, add_block_with_tx_proposal, get_test_ledger,
            manually_sync_account, setup_wallet_service, MOB,
        },
        util::b58::b58_encode_public_address,
    };
    use mc_account_keys::{AccountKey, PublicAddress};
    use mc_common::logger::{test_with_logger, Logger};
    use mc_crypto_keys::{ReprBytes, RistrettoPrivate, RistrettoPublic};
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
        let mut confirmation_bytes = [0u8; 32];
        rng.fill_bytes(&mut confirmation_bytes);
        let confirmation_number = TxOutConfirmationNumber::from(confirmation_bytes);

        let mut proto_tx_receipt = mc_api::external::Receipt::new();
        proto_tx_receipt.set_public_key((&txo.public_key).into());
        proto_tx_receipt.set_tombstone_block(tombstone);
        let mut proto_confirmation = mc_api::external::TxOutConfirmationNumber::new();
        proto_confirmation.set_hash(confirmation_number.to_vec());
        proto_tx_receipt.set_confirmation(proto_confirmation);
        let mut proto_commitment = mc_api::external::CompressedRistretto::new();
        proto_commitment.set_data(txo.amount.commitment.to_bytes().to_vec());
        let mut proto_amount = mc_api::external::Amount::new();
        proto_amount.set_commitment(proto_commitment);
        proto_amount.set_masked_value(txo.amount.masked_value);
        proto_tx_receipt.set_amount(proto_amount);

        let tx_receipt =
            ReceiverReceipt::try_from(&proto_tx_receipt).expect("Could not convert tx receipt");
        assert_eq!(txo.public_key, tx_receipt.public_key);
        assert_eq!(tombstone, tx_receipt.tombstone_block);
        assert_eq!(confirmation_number, tx_receipt.confirmation);
        assert_eq!(txo.amount, tx_receipt.amount);
    }

    #[test_with_logger]
    fn test_create_receipt(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        let service = setup_wallet_service(ledger_db.clone(), logger.clone());
        let alice = service
            .create_account(Some("Alice's Main Account".to_string()))
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
            .create_account(Some("Bob's Main Account".to_string()))
            .unwrap();
        let bob_addresses = service
            .get_addresses_for_account(&AccountID(bob.account_id_hex.clone()), None, None)
            .expect("Could not get addresses for Bob");
        let bob_address = bob_addresses[0].assigned_subaddress_b58.clone();

        // Create a TxProposal to Bob
        let tx_proposal = service
            .build_transaction(
                &alice.account_id_hex,
                &vec![(bob_address.to_string(), (24 * MOB).to_string())],
                None,
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
            &alice.account_id_hex,
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
            .list_txos(&AccountID(bob.account_id_hex), None, None)
            .expect("Could not get Bob Txos");
        assert_eq!(txos.len(), 1);

        // Get the corresponding TransactionLog for Alice's Account - only the sender
        // has the confirmation number.
        let transaction_logs = service
            .list_transaction_logs(&AccountID(alice.account_id_hex), None, None)
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

        let confirmations = service
            .get_confirmations(&sent_transaction_log.transaction_id_hex)
            .expect("Could not get confirmations");
        assert_eq!(confirmations.len(), 1);

        let txo_pubkey =
            mc_util_serial::decode(&txos[0].public_key).expect("Could not decode pubkey");
        assert_eq!(receipt.public_key, txo_pubkey);
        assert_eq!(receipt.tombstone_block, 63); // Ledger seeded with 12 blocks at tx construction, then one appended + 50
        let txo: TxOut = mc_util_serial::decode(&txos[0].txo).expect("Could not decode txo");
        assert_eq!(receipt.amount, txo.amount);
        assert_eq!(receipt.confirmation, confirmations[0].confirmation);
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
            .create_account(Some("Alice's Main Account".to_string()))
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
            .create_account(Some("Bob's Main Account".to_string()))
            .unwrap();
        let bob_addresses = service
            .get_addresses_for_account(&AccountID(bob.account_id_hex.clone()), None, None)
            .expect("Could not get addresses for Bob");
        let bob_address = &bob_addresses[0].assigned_subaddress_b58.clone();

        // Create a TxProposal to Bob
        let tx_proposal = service
            .build_transaction(
                &alice.account_id_hex,
                &vec![(bob_address.to_string(), (24 * MOB).to_string())],
                None,
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
        let (status, _txo) = service
            .check_receipt_status(&bob_address, &receipt)
            .expect("Could not check status of receipt");

        // Status should be pending until block lands and is scanned
        assert_eq!(status, ReceiptTransactionStatus::TransactionPending);

        // Land the Txo in the ledger - only sync for the sender
        TransactionLog::log_submitted(
            tx_proposal.clone(),
            14,
            "".to_string(),
            &alice.account_id_hex,
            &service.wallet_db.get_conn().unwrap(),
        )
        .expect("Could not log submitted");

        // Status for Bob should still be pending, even though the Txos will show up in
        // the wallet, but under Alice's account.
        let (status, _txo) = service
            .check_receipt_status(&bob_address, &receipt)
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
        let (status, _txo) = service
            .check_receipt_status(&bob_address, &receipt)
            .expect("Could not check status of receipt");
        assert_eq!(status, ReceiptTransactionStatus::TransactionSuccess);

        // Status for Alice would be pending, because she never received (and never will
        // receive) the Txos.
        let alice_address = &b58_encode_public_address(&alice_public_address)
            .expect("Could not encode Alice address");
        let (status, _txo) = service
            .check_receipt_status(&alice_address, &receipt)
            .expect("Could not check status of receipt");
        assert_eq!(status, ReceiptTransactionStatus::TransactionPending);

        // assert_eq!(status, ReceiptTransactionStatus::FailedAmountDecryption);
    }

    #[test_with_logger]
    fn test_check_receiver_receipt_status_wrong_value(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        let service = setup_wallet_service(ledger_db.clone(), logger.clone());
        let alice = service
            .create_account(Some("Alice's Main Account".to_string()))
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
            .create_account(Some("Bob's Main Account".to_string()))
            .unwrap();
        let bob_addresses = service
            .get_addresses_for_account(&AccountID(bob.account_id_hex.clone()), None, None)
            .expect("Could not get addresses for Bob");
        let bob_address = &bob_addresses[0].assigned_subaddress_b58.clone();
        let bob_account_id = AccountID(bob.account_id_hex.to_string());

        // Create a TxProposal to Bob
        let tx_proposal0 = service
            .build_transaction(
                &alice.account_id_hex,
                &vec![(bob_address.to_string(), (24 * MOB).to_string())],
                None,
                None,
                None,
                None,
                None,
            )
            .expect("Could not build transaction");

        let receipts = service
            .create_receiver_receipts(&tx_proposal0)
            .expect("Could not create receiver receipt");
        let mut receipt0 = receipts[0].clone();

        // Land the Txo in the ledger - only sync for the sender
        TransactionLog::log_submitted(
            tx_proposal0.clone(),
            14,
            "".to_string(),
            &alice.account_id_hex,
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

        // Bob checks the status, and is expecting an incorrect value, from a
        // transaction with a different shared secret
        receipt0.amount = Amount::new(18 * MOB as u64, &RistrettoPublic::from_random(&mut rng))
            .expect("Could not create Amount");
        let (status, _txo) = service
            .check_receipt_status(&bob_address, &receipt0)
            .expect("Could not check status of receipt");
        assert_eq!(status, ReceiptTransactionStatus::FailedAmountDecryption);

        // Now check status with a correct shared secret, but the wrong value
        let bob_account_key: AccountKey = mc_util_serial::decode(
            &Account::get(&bob_account_id, &service.wallet_db.get_conn().unwrap())
                .expect("Could not get bob account")
                .account_key,
        )
        .expect("Could not decode");
        let public_key: RistrettoPublic = RistrettoPublic::try_from(&receipt0.public_key)
            .expect("Could not get ristretto public from compressed");
        let shared_secret =
            get_tx_out_shared_secret(bob_account_key.view_private_key(), &public_key);
        receipt0.amount =
            Amount::new(18 * MOB as u64, &shared_secret).expect("Could not create Amount");
        let (status, _txo) = service
            .check_receipt_status(&bob_address, &receipt0)
            .expect("Could not check status of receipt");
        assert_eq!(
            status,
            ReceiptTransactionStatus::AmountMismatch(
                "Expected: 18000000000000, Got: 24000000000000".to_string()
            )
        );

        // Status for Alice would be pending, because she never received (and
        // never will receive) the Txos.
        let alice_address = &b58_encode_public_address(&alice_public_address)
            .expect("Could not encode alice address");
        let (status, _txo) = service
            .check_receipt_status(&alice_address, &receipt0)
            .expect("Could not check status of receipt");
        assert_eq!(status, ReceiptTransactionStatus::TransactionPending);

        // assert_eq!(status, ReceiptTransactionStatus::FailedAmountDecryption);
    }

    #[test_with_logger]
    fn test_check_receiver_receipt_status_invalid_confirmation(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        let service = setup_wallet_service(ledger_db.clone(), logger.clone());
        let alice = service
            .create_account(Some("Alice's Main Account".to_string()))
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
            .create_account(Some("Bob's Main Account".to_string()))
            .unwrap();
        let bob_addresses = service
            .get_addresses_for_account(&AccountID(bob.account_id_hex.clone()), None, None)
            .expect("Could not get addresses for Bob");
        let bob_address = &bob_addresses[0].assigned_subaddress_b58.clone();
        let bob_account_id = AccountID(bob.account_id_hex.to_string());

        // Create a TxProposal to Bob
        let tx_proposal0 = service
            .build_transaction(
                &alice.account_id_hex,
                &vec![(bob_address.to_string(), (24 * MOB).to_string())],
                None,
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
            &alice.account_id_hex,
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

        // Construct an invalid receipt with an incorrect confirmation number.
        let mut receipt = receipt0.clone();
        let mut bad_confirmation_bytes = [0u8; 32];
        rng.fill_bytes(&mut bad_confirmation_bytes);
        let bad_confirmation = TxOutConfirmationNumber::from(bad_confirmation_bytes);
        receipt.confirmation = bad_confirmation;

        // Bob checks the status, and is expecting an incorrect value
        let (status, _txo) = service
            .check_receipt_status(&bob_address, &receipt)
            .expect("Could not check status of receipt");
        assert_eq!(status, ReceiptTransactionStatus::InvalidConfirmation);

        // Checking for the sender will be pending because the Txos haven't
        // landed for alice (and never will).
        let alice_address = &b58_encode_public_address(&alice_public_address)
            .expect("Could not encode alice address");
        let (status, _txo) = service
            .check_receipt_status(&alice_address, &receipt)
            .expect("Could not check status of receipt");
        assert_eq!(status, ReceiptTransactionStatus::TransactionPending);

        // assert_eq!(status, ReceiptTransactionStatus::FailedAmountDecryption);
    }
}
