// Copyright (c) 2020-2021 MobileCoin Inc.

//! Service for managing transactions.

use crate::{
    db::{
        b58_decode,
        models::TransactionLog,
        transaction_log::{AssociatedTxos, TransactionLogModel},
        WalletDbError,
    },
    error::WalletTransactionBuilderError,
    service::{transaction_builder::WalletTransactionBuilder, WalletService},
};
use mc_common::logger::log;
use mc_connection::{BlockchainConnection, RetryableUserTxConnection, UserTxConnection};
use mc_fog_report_validation::FogPubkeyResolver;
use mc_mobilecoind::payments::TxProposal;

use crate::service::address::{AddressService, AddressServiceError};
use displaydoc::Display;
use std::{convert::TryFrom, iter::empty, sync::atomic::Ordering};

/// Errors for the Transaction Service.
#[derive(Display, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum TransactionServiceError {
    /// Error interacting with the database: {0}
    Database(WalletDbError),

    /// Error building transaction: {0}
    TransactionBuilder(WalletTransactionBuilderError),

    /// Error parsing u64
    U64Parse,

    /// Submit transaction expected an account to produce a transaction log on
    /// submit.
    MissingAccountOnSubmit,

    /// Node not found.
    NodeNotFound,

    /// No peers configured.
    NoPeersConfigured,

    /// Error converting to/from API protos: {0}
    ProtoConversion(mc_api::ConversionError),

    /// Error Converting Proto but throws convert::Infallible.
    ProtoConversionInfallible,

    /// Cannot complete this action in offline mode.
    Offline,

    /// Connection Error
    Connection(retry::Error<mc_connection::Error>),

    /// Invalid Public Address: {0}
    InvalidPublicAddress(String),

    /// Address Service Error: {0}
    AddressService(AddressServiceError),
}

impl From<WalletDbError> for TransactionServiceError {
    fn from(src: WalletDbError) -> Self {
        Self::Database(src)
    }
}

impl From<std::num::ParseIntError> for TransactionServiceError {
    fn from(_src: std::num::ParseIntError) -> Self {
        Self::U64Parse
    }
}

impl From<WalletTransactionBuilderError> for TransactionServiceError {
    fn from(src: WalletTransactionBuilderError) -> Self {
        Self::TransactionBuilder(src)
    }
}

impl From<mc_api::ConversionError> for TransactionServiceError {
    fn from(src: mc_api::ConversionError) -> Self {
        Self::ProtoConversion(src)
    }
}

impl From<retry::Error<mc_connection::Error>> for TransactionServiceError {
    fn from(e: retry::Error<mc_connection::Error>) -> Self {
        Self::Connection(e)
    }
}

impl From<AddressServiceError> for TransactionServiceError {
    fn from(e: AddressServiceError) -> Self {
        Self::AddressService(e)
    }
}

/// Trait defining the ways in which the wallet can interact with and manage
/// transactions.
pub trait TransactionService {
    /// Builds a transaction from the given account to the specified recipients.
    fn build_transaction(
        &self,
        account_id_hex: &str,
        addresses_and_values: &[(String, String)],
        input_txo_ids: Option<&Vec<String>>,
        fee: Option<String>,
        tombstone_block: Option<String>,
        max_spendable_value: Option<String>,
    ) -> Result<TxProposal, TransactionServiceError>;

    /// Submits a pre-built TxProposal to the MobileCoin Consensus Network.
    fn submit_transaction(
        &self,
        tx_proposal: TxProposal,
        comment: Option<String>,
        account_id_hex: Option<String>,
    ) -> Result<Option<(TransactionLog, AssociatedTxos)>, TransactionServiceError>;

    /// Convenience method that builds and submits in one go.
    #[allow(clippy::too_many_arguments)]
    fn build_and_submit(
        &self,
        account_id_hex: &str,
        addresses_and_values: &[(String, String)],
        input_txo_ids: Option<&Vec<String>>,
        fee: Option<String>,
        tombstone_block: Option<String>,
        max_spendable_value: Option<String>,
        comment: Option<String>,
    ) -> Result<(TransactionLog, AssociatedTxos), TransactionServiceError>;
}

impl<T, FPR> TransactionService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn build_transaction(
        &self,
        account_id_hex: &str,
        addresses_and_values: &[(String, String)],
        input_txo_ids: Option<&Vec<String>>,
        fee: Option<String>,
        tombstone_block: Option<String>,
        max_spendable_value: Option<String>,
    ) -> Result<TxProposal, TransactionServiceError> {
        let mut builder = WalletTransactionBuilder::new(
            account_id_hex.to_string(),
            self.wallet_db.clone(),
            self.ledger_db.clone(),
            self.fog_resolver_factory.clone(),
            self.logger.clone(),
        );

        for (recipient_public_address, value) in addresses_and_values {
            if !self.verify_address(recipient_public_address)? {
                return Err(TransactionServiceError::InvalidPublicAddress(
                    recipient_public_address.to_string(),
                ));
            };
            let recipient = b58_decode(recipient_public_address)?;
            builder.add_recipient(recipient, value.parse::<u64>()?)?;
        }

        if let Some(inputs) = input_txo_ids {
            builder.set_txos(inputs)?;
        } else {
            let max_spendable = if let Some(msv) = max_spendable_value {
                Some(msv.parse::<u64>()?)
            } else {
                None
            };
            builder.select_txos(max_spendable)?;
        }
        if let Some(tombstone) = tombstone_block {
            builder.set_tombstone(tombstone.parse::<u64>()?)?;
        } else {
            builder.set_tombstone(0)?;
        }
        if let Some(f) = fee {
            builder.set_fee(f.parse::<u64>()?)?;
        }
        let tx_proposal = builder.build()?;

        // FIXME: WS-32 - Might be nice to have a tx_proposal table so that you don't
        // have to write these out to local files.
        Ok(tx_proposal)
    }

    fn submit_transaction(
        &self,
        tx_proposal: TxProposal,
        comment: Option<String>,
        account_id_hex: Option<String>,
    ) -> Result<Option<(TransactionLog, AssociatedTxos)>, TransactionServiceError> {
        if self.offline {
            return Err(TransactionServiceError::Offline);
        }

        // Pick a peer to submit to.
        let responder_ids = self.peer_manager.responder_ids();
        if responder_ids.is_empty() {
            return Err(TransactionServiceError::NoPeersConfigured);
        }

        let idx = self.submit_node_offset.fetch_add(1, Ordering::SeqCst);
        let responder_id = &responder_ids[idx % responder_ids.len()];

        // FIXME: WS-34 - would prefer not to convert to proto as intermediary
        let tx_proposal_proto = mc_mobilecoind_api::TxProposal::try_from(&tx_proposal)
            .map_err(|_| TransactionServiceError::ProtoConversionInfallible)?;

        // Try to submit.
        let tx = mc_transaction_core::tx::Tx::try_from(tx_proposal_proto.get_tx())
            .map_err(|_| TransactionServiceError::ProtoConversionInfallible)?;

        let block_index = self
            .peer_manager
            .conn(responder_id)
            .ok_or(TransactionServiceError::NodeNotFound)?
            .propose_tx(&tx, empty())
            .map_err(TransactionServiceError::from)?;

        // Log the transaction.
        let result = if let Some(a) = account_id_hex {
            let transaction_log = TransactionLog::log_submitted(
                tx_proposal,
                block_index,
                comment.unwrap_or_else(|| "".to_string()),
                &a,
                &self.wallet_db.get_conn()?,
            )?;
            let associated_txos =
                transaction_log.get_associated_txos(&self.wallet_db.get_conn()?)?;
            Ok(Some((transaction_log, associated_txos)))
        } else {
            Ok(None)
        };

        log::trace!(
            self.logger,
            "Tx {:?} submitted at block height {}",
            tx,
            block_index
        );

        result
    }

    fn build_and_submit(
        &self,
        account_id_hex: &str,
        addresses_and_values: &[(String, String)],
        input_txo_ids: Option<&Vec<String>>,
        fee: Option<String>,
        tombstone_block: Option<String>,
        max_spendable_value: Option<String>,
        comment: Option<String>,
    ) -> Result<(TransactionLog, AssociatedTxos), TransactionServiceError> {
        let tx_proposal = self.build_transaction(
            account_id_hex,
            &addresses_and_values,
            input_txo_ids,
            fee,
            tombstone_block,
            max_spendable_value,
        )?;
        if let Some(transaction_log_and_associated_txos) =
            self.submit_transaction(tx_proposal, comment, Some(account_id_hex.to_string()))?
        {
            Ok(transaction_log_and_associated_txos)
        } else {
            Err(TransactionServiceError::MissingAccountOnSubmit)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db::{
            account::AccountID,
            b58_encode,
            models::Txo,
            txo::{TxoDetails, TxoModel},
        },
        service::{account::AccountService, address::AddressService, balance::BalanceService},
        test_utils::{
            add_block_from_transaction_log, add_block_to_ledger_db, get_test_ledger,
            setup_wallet_service, wait_for_sync, MOB,
        },
    };
    use mc_account_keys::{AccountKey, PublicAddress};
    use mc_common::logger::{test_with_logger, Logger};
    use mc_crypto_rand::rand_core::RngCore;
    use mc_transaction_core::ring_signature::KeyImage;
    use rand::{rngs::StdRng, SeedableRng};

    // Test sending a transaction from Alice -> Bob, and then from Bob -> Alice
    #[test_with_logger]
    fn test_send_transaction(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        let service = setup_wallet_service(ledger_db.clone(), logger.clone());

        // Create our main account for the wallet
        let alice = service
            .create_account(Some("Alice's Main Account".to_string()))
            .unwrap();

        // Add a block with a transaction for Alice
        let alice_account_key: AccountKey = mc_util_serial::decode(&alice.account_key).unwrap();
        let alice_account_id = AccountID::from(&alice_account_key);
        let alice_public_address = alice_account_key.subaddress(alice.main_subaddress_index as u64);
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![alice_public_address.clone()],
            100 * MOB as u64,
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        wait_for_sync(&ledger_db, &service.wallet_db, &alice_account_id, 13);

        // Verify balance for Alice
        let balance = service
            .get_balance_for_account(&AccountID(alice.account_id_hex.clone()))
            .unwrap();
        assert_eq!(balance.unspent, 100 * MOB as u128);

        // Add an account for Bob
        let bob = service
            .create_account(Some("Bob's Main Account".to_string()))
            .unwrap();
        let bob_account_key: AccountKey =
            mc_util_serial::decode(&bob.account_key).expect("Could not decode account key");
        let bob_account_id = AccountID::from(&bob_account_key);

        // Create an assigned subaddress for Bob
        let bob_address_from_alice = service
            .assign_address_for_account(&AccountID(bob.account_id_hex.clone()), Some("From Alice"))
            .unwrap();

        // Send a transaction from Alice to Bob
        let (transaction_log, _associated_txos) = service
            .build_and_submit(
                &alice.account_id_hex,
                &[(
                    bob_address_from_alice.assigned_subaddress_b58,
                    (42 * MOB).to_string(),
                )],
                None,
                None,
                None,
                None,
                None,
            )
            .unwrap();
        log::info!(logger, "Built and submitted transaction from Alice");

        // NOTE: Submitting to the test ledger via propose_tx doesn't actually add the
        // block to the ledger, because no consensus is occurring, so this is the
        // workaround.
        {
            log::info!(logger, "Adding block from transaction log");
            let conn = service.wallet_db.get_conn().unwrap();
            add_block_from_transaction_log(&mut ledger_db, &conn, &transaction_log);
        }

        wait_for_sync(&ledger_db, &service.wallet_db, &alice_account_id, 14);
        wait_for_sync(&ledger_db, &service.wallet_db, &bob_account_id, 14);

        // Get the Txos from the transaction log
        let transaction_txos = transaction_log
            .get_associated_txos(&service.wallet_db.get_conn().unwrap())
            .unwrap();
        let secreted = transaction_txos
            .outputs
            .iter()
            .map(|t| Txo::get(&t.txo_id_hex, &service.wallet_db.get_conn().unwrap()).unwrap())
            .collect::<Vec<TxoDetails>>();
        assert_eq!(secreted.len(), 1);
        assert_eq!(secreted[0].txo.value, 42 * MOB);

        let change = transaction_txos
            .change
            .iter()
            .map(|t| Txo::get(&t.txo_id_hex, &service.wallet_db.get_conn().unwrap()).unwrap())
            .collect::<Vec<TxoDetails>>();
        assert_eq!(change.len(), 1);
        assert_eq!(change[0].txo.value, (57.99 * MOB as f64) as i64);

        let inputs = transaction_txos
            .inputs
            .iter()
            .map(|t| Txo::get(&t.txo_id_hex, &service.wallet_db.get_conn().unwrap()).unwrap())
            .collect::<Vec<TxoDetails>>();
        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0].txo.value, 100 * MOB);

        // Verify balance for Alice = original balance - fee - txo_value
        let balance = service
            .get_balance_for_account(&AccountID(alice.account_id_hex.clone()))
            .unwrap();
        assert_eq!(balance.unspent, 57990000000000);

        // Bob's balance should be = output_txo_value
        let bob_balance = service
            .get_balance_for_account(&AccountID(bob.account_id_hex.clone()))
            .unwrap();
        assert_eq!(bob_balance.unspent, 42000000000000);

        // Bob should now be able to send to Alice
        let (transaction_log, _associated_txos) = service
            .build_and_submit(
                &bob.account_id_hex,
                &[(
                    b58_encode(&alice_public_address).unwrap(),
                    (8 * MOB).to_string(),
                )],
                None,
                None,
                None,
                None,
                None,
            )
            .unwrap();

        // NOTE: Submitting to the test ledger via propose_tx doesn't actually add the
        // block to the ledger, because no consensus is occurring, so this is the
        // workaround.

        {
            log::info!(logger, "Adding block from transaction log");
            let conn = service.wallet_db.get_conn().unwrap();
            add_block_from_transaction_log(&mut ledger_db, &conn, &transaction_log);
        }

        wait_for_sync(&ledger_db, &service.wallet_db, &alice_account_id, 15);
        wait_for_sync(&ledger_db, &service.wallet_db, &bob_account_id, 15);

        let alice_balance = service
            .get_balance_for_account(&AccountID(alice.account_id_hex))
            .unwrap();
        assert_eq!(alice_balance.unspent, 65990000000000);

        // Bob's balance should be = output_txo_value
        let bob_balance = service
            .get_balance_for_account(&AccountID(bob.account_id_hex))
            .unwrap();
        assert_eq!(bob_balance.unspent, 33990000000000);
    }

    // Building a transaction for an invalid public address should fail.
    #[test_with_logger]
    fn test_invalid_public_address_fails(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        let service = setup_wallet_service(ledger_db.clone(), logger.clone());

        // Create our main account for the wallet
        let alice = service
            .create_account(Some("Alice's Main Account".to_string()))
            .unwrap();

        // Add a block with a transaction for Alice
        let alice_account_key: AccountKey = mc_util_serial::decode(&alice.account_key).unwrap();
        let alice_account_id = AccountID::from(&alice_account_key);
        let alice_public_address = alice_account_key.subaddress(alice.main_subaddress_index as u64);
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![alice_public_address.clone()],
            100 * MOB as u64,
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        wait_for_sync(&ledger_db, &service.wallet_db, &alice_account_id, 13);

        match service.build_transaction(
            &alice.account_id_hex,
            &vec![("NOTB58".to_string(), (42 * MOB).to_string())],
            None,
            None,
            None,
            None,
        ) {
            Ok(_) => {
                panic!("Should not be able to build transaction to invalid b58 public address")
            }
            Err(TransactionServiceError::InvalidPublicAddress(_)) => {}
            Err(e) => panic!("Unexpected error {:?}", e),
        };
    }

    // FIXME: Test with 0 change transactions
    // FIXME: Test with balance > u64::max
    // FIXME: sending a transaction with value > u64::max
}
