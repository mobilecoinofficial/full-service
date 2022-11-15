// Copyright (c) 2020-2021 MobileCoin Inc.

//! Service for managing transactions.

use crate::{
    db::{
        account::{AccountID, AccountModel},
        models::{Account, TransactionLog},
        transaction,
        transaction_log::{AssociatedTxos, TransactionLogModel, ValueMap},
        WalletDbError,
    },
    error::WalletTransactionBuilderError,
    json_rpc::v2::models::amount::Amount as AmountJSON,
    service::{
        ledger::{LedgerService, LedgerServiceError},
        models::tx_proposal::TxProposal,
        transaction_builder::WalletTransactionBuilder,
        WalletService,
    },
    util::b58::{b58_decode_public_address, B58Error},
};
use mc_account_keys::AccountKey;
use mc_blockchain_types::BlockVersion;
use mc_common::logger::log;
use mc_connection::{BlockchainConnection, RetryableUserTxConnection, UserTxConnection};
use mc_fog_report_validation::FogPubkeyResolver;
use mc_transaction_builder::{
    BurnRedemptionMemoBuilder, EmptyMemoBuilder, MemoBuilder, RTHMemoBuilder,
};
use mc_transaction_core::{
    constants::{MAX_INPUTS, MAX_OUTPUTS},
    tokens::Mob,
    Amount, Token, TokenId,
};
use mc_transaction_extra::{BurnRedemptionMemo, SenderMemoCredential};

use crate::service::address::{AddressService, AddressServiceError};
use displaydoc::Display;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use std::{convert::TryFrom, iter::empty, sync::atomic::Ordering};

use super::models::tx_proposal::UnsignedTxProposal;

/// Errors for the Transaction Service.
#[derive(Display, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum TransactionServiceError {
    ///Error interacting with the B58 Util: {0}
    B58(B58Error),

    /// Error interacting with the database: {0}
    Database(WalletDbError),

    /// Error building transaction: {0}
    TransactionBuilder(WalletTransactionBuilderError),

    /// Error parsing u64
    U64Parse,

    /** Submit transaction expected an account to produce a transaction log
     * on submit.
     */
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

    /// Diesel Error: {0}
    Diesel(diesel::result::Error),

    /// Ledger DB Error: {0}
    LedgerDB(mc_ledger_db::Error),

    /// Invalid Amount: {0}
    InvalidAmount(String),

    /// No default fee found for token id: {0}
    DefaultFeeNotFoundForToken(TokenId),

    /// Error decoding hex string
    FromHex(hex::FromHexError),

    /// Invalid burn redemption memo: {0}
    InvalidBurnRedemptionMemo(String),

    /// mc_util_serial decode error: {0}
    Decode(mc_util_serial::DecodeError),

    /// Tx Builder Error: {0}
    TxBuilder(mc_transaction_builder::TxBuilderError),

    /// Ledger service error: {0}
    LedgerService(LedgerServiceError),

    /// Key Error: {0}
    Key(mc_crypto_keys::KeyError),

    /// RetryError
    Retry(mc_connection::RetryError<mc_connection::Error>),

    /// Ring CT Error: {0}
    RingCT(mc_transaction_core::ring_ct::Error),
}

impl From<WalletDbError> for TransactionServiceError {
    fn from(src: WalletDbError) -> Self {
        Self::Database(src)
    }
}

impl From<B58Error> for TransactionServiceError {
    fn from(src: B58Error) -> Self {
        Self::B58(src)
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

impl From<diesel::result::Error> for TransactionServiceError {
    fn from(src: diesel::result::Error) -> Self {
        Self::Diesel(src)
    }
}

impl From<mc_ledger_db::Error> for TransactionServiceError {
    fn from(src: mc_ledger_db::Error) -> Self {
        Self::LedgerDB(src)
    }
}

impl From<hex::FromHexError> for TransactionServiceError {
    fn from(src: hex::FromHexError) -> Self {
        Self::FromHex(src)
    }
}

impl From<mc_util_serial::DecodeError> for TransactionServiceError {
    fn from(src: mc_util_serial::DecodeError) -> Self {
        Self::Decode(src)
    }
}

impl From<mc_transaction_builder::TxBuilderError> for TransactionServiceError {
    fn from(src: mc_transaction_builder::TxBuilderError) -> Self {
        Self::TxBuilder(src)
    }
}

impl From<mc_crypto_keys::KeyError> for TransactionServiceError {
    fn from(src: mc_crypto_keys::KeyError) -> Self {
        Self::Key(src)
    }
}

impl From<LedgerServiceError> for TransactionServiceError {
    fn from(src: LedgerServiceError) -> Self {
        Self::LedgerService(src)
    }
}

impl From<mc_connection::RetryError<mc_connection::Error>> for TransactionServiceError {
    fn from(src: mc_connection::RetryError<mc_connection::Error>) -> Self {
        Self::Retry(src)
    }
}

impl From<mc_transaction_core::ring_ct::Error> for TransactionServiceError {
    fn from(src: mc_transaction_core::ring_ct::Error) -> Self {
        Self::RingCT(src)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
/// This represents the different types of Transaction Memos that can be used in
/// a given transaction
///
/// * Empty
///
/// * RTH
///
/// * BurnRedemption
pub enum TransactionMemo {
    /// Empty Transaction Memo.
    Empty,

    /// Recoverable Transaction History memo with an optional u64 specifying the
    /// subaddress index to generate the sender memo credential from.
    RTH(Option<u64>),

    /// Burn Redemption memo, with an optional 64 byte redemption memo hex
    /// string.
    #[serde(with = "BigArray")]
    BurnRedemption([u8; BurnRedemptionMemo::MEMO_DATA_LEN]),
}

impl TransactionMemo {
    pub fn memo_builder(
        &self,
        account_key: Option<AccountKey>,
    ) -> Result<Box<dyn MemoBuilder + Send + Sync>, WalletTransactionBuilderError> {
        match self {
            Self::Empty => Ok(Box::new(EmptyMemoBuilder::default())),
            Self::RTH(subaddress_index) => {
                let mut memo_builder = RTHMemoBuilder::default();
                let account_key = account_key
                    .ok_or(WalletTransactionBuilderError::RTHUnavailableForViewOnlyAccounts)?;
                let sender_memo_credential = match subaddress_index {
                    Some(subaddress_index) => {
                        SenderMemoCredential::new_from_address_and_spend_private_key(
                            &account_key.subaddress(*subaddress_index),
                            account_key.subaddress_spend_private(*subaddress_index),
                        )
                    }
                    None => SenderMemoCredential::from(&account_key),
                };
                memo_builder.set_sender_credential(sender_memo_credential);
                memo_builder.enable_destination_memo();
                Ok(Box::new(memo_builder))
            }
            Self::BurnRedemption(memo_data) => {
                let mut memo_builder = BurnRedemptionMemoBuilder::new(*memo_data);
                memo_builder.enable_destination_memo();
                Ok(Box::new(memo_builder))
            }
        }
    }
}

/// Trait defining the ways in which the wallet can interact with and manage
/// transactions.
pub trait TransactionService {
    #[allow(clippy::too_many_arguments)]
    fn build_transaction(
        &self,
        account_id_hex: &str,
        addresses_and_amounts: &[(String, AmountJSON)],
        input_txo_ids: Option<&Vec<String>>,
        fee_value: Option<String>,
        fee_token_id: Option<String>,
        tombstone_block: Option<String>,
        max_spendable_value: Option<String>,
        memo: TransactionMemo,
        block_version: Option<BlockVersion>,
    ) -> Result<UnsignedTxProposal, TransactionServiceError>;

    #[allow(clippy::too_many_arguments)]
    fn build_and_sign_transaction(
        &self,
        account_id_hex: &str,
        addresses_and_amounts: &[(String, AmountJSON)],
        input_txo_ids: Option<&Vec<String>>,
        fee_value: Option<String>,
        fee_token_id: Option<String>,
        tombstone_block: Option<String>,
        max_spendable_value: Option<String>,
        memo: TransactionMemo,
        block_version: Option<BlockVersion>,
    ) -> Result<TxProposal, TransactionServiceError>;

    /// Submits a pre-built TxProposal to the MobileCoin Consensus Network.
    fn submit_transaction(
        &self,
        tx_proposal: &TxProposal,
        comment: Option<String>,
        account_id_hex: Option<String>,
    ) -> Result<Option<(TransactionLog, AssociatedTxos, ValueMap)>, TransactionServiceError>;

    #[allow(clippy::too_many_arguments)]
    fn build_sign_and_submit_transaction(
        &self,
        account_id_hex: &str,
        addresses_and_amounts: &[(String, AmountJSON)],
        input_txo_ids: Option<&Vec<String>>,
        fee_value: Option<String>,
        fee_token_id: Option<String>,
        tombstone_block: Option<String>,
        max_spendable_value: Option<String>,
        comment: Option<String>,
        memo: TransactionMemo,
        block_version: Option<BlockVersion>,
    ) -> Result<(TransactionLog, AssociatedTxos, ValueMap, TxProposal), TransactionServiceError>;
}

impl<T, FPR> TransactionService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn build_transaction(
        &self,
        account_id_hex: &str,
        addresses_and_amounts: &[(String, AmountJSON)],
        input_txo_ids: Option<&Vec<String>>,
        fee_value: Option<String>,
        fee_token_id: Option<String>,
        tombstone_block: Option<String>,
        max_spendable_value: Option<String>,
        memo: TransactionMemo,
        block_version: Option<BlockVersion>,
    ) -> Result<UnsignedTxProposal, TransactionServiceError> {
        validate_number_inputs(input_txo_ids.unwrap_or(&Vec::new()).len() as u64)?;
        validate_number_outputs(addresses_and_amounts.len() as u64)?;

        let conn = self.get_conn()?;
        transaction(&conn, || {
            let mut builder = WalletTransactionBuilder::new(
                account_id_hex.to_string(),
                self.ledger_db.clone(),
                self.fog_resolver_factory.clone(),
            );

            let mut default_fee_token_id = Mob::ID;

            for (recipient_public_address, amount) in addresses_and_amounts {
                if !self.verify_address(recipient_public_address)? {
                    return Err(TransactionServiceError::InvalidPublicAddress(
                        recipient_public_address.to_string(),
                    ));
                };
                let recipient = b58_decode_public_address(recipient_public_address)?;
                let amount =
                    Amount::try_from(amount).map_err(TransactionServiceError::InvalidAmount)?;
                builder.add_recipient(recipient, amount.value, amount.token_id)?;
                default_fee_token_id = amount.token_id;
            }

            if let Some(tombstone) = tombstone_block {
                builder.set_tombstone(tombstone.parse::<u64>()?)?;
            } else {
                builder.set_tombstone(0)?;
            }

            let fee_token_id = match fee_token_id {
                Some(t) => TokenId::from(t.parse::<u64>()?),
                None => default_fee_token_id,
            };

            let fee_value = match fee_value {
                Some(f) => f.parse::<u64>()?,
                None => *self.get_network_fees()?.get(&fee_token_id).ok_or(
                    TransactionServiceError::DefaultFeeNotFoundForToken(fee_token_id),
                )?,
            };

            builder.set_fee(fee_value, fee_token_id)?;

            match block_version {
                Some(v) => builder.set_block_version(v),
                None => builder.set_block_version(self.get_network_block_version()?),
            }

            if let Some(inputs) = input_txo_ids {
                builder.set_txos(&conn, inputs)?;
            } else {
                let max_spendable = if let Some(msv) = max_spendable_value {
                    Some(msv.parse::<u64>()?)
                } else {
                    None
                };
                builder.select_txos(&conn, max_spendable)?;
            }

            let unsigned_tx_proposal = builder.build(memo, &conn)?;

            Ok(unsigned_tx_proposal)
        })
    }

    fn build_and_sign_transaction(
        &self,
        account_id_hex: &str,
        addresses_and_amounts: &[(String, AmountJSON)],
        input_txo_ids: Option<&Vec<String>>,
        fee_value: Option<String>,
        fee_token_id: Option<String>,
        tombstone_block: Option<String>,
        max_spendable_value: Option<String>,
        memo: TransactionMemo,
        block_version: Option<BlockVersion>,
    ) -> Result<TxProposal, TransactionServiceError> {
        let unsigned_tx_proposal = self.build_transaction(
            account_id_hex,
            addresses_and_amounts,
            input_txo_ids,
            fee_value,
            fee_token_id,
            tombstone_block,
            max_spendable_value,
            memo,
            block_version,
        )?;
        let conn = self.get_conn()?;
        transaction(&conn, || {
            let account = Account::get(&AccountID(account_id_hex.to_string()), &conn)?;
            let account_key: AccountKey = mc_util_serial::decode(&account.account_key)?;
            let tx_proposal = unsigned_tx_proposal.sign(&account_key)?;

            TransactionLog::log_built(tx_proposal.clone(), "".to_string(), account_id_hex, &conn)?;

            Ok(tx_proposal)
        })
    }

    fn submit_transaction(
        &self,
        tx_proposal: &TxProposal,
        comment: Option<String>,
        account_id_hex: Option<String>,
    ) -> Result<Option<(TransactionLog, AssociatedTxos, ValueMap)>, TransactionServiceError> {
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

        let block_index = self
            .peer_manager
            .conn(responder_id)
            .ok_or(TransactionServiceError::NodeNotFound)?
            .propose_tx(&tx_proposal.tx, empty())
            .map_err(TransactionServiceError::from)?;

        log::trace!(
            self.logger,
            "Tx {:?} submitted at block height {}",
            tx_proposal.tx,
            block_index
        );

        if let Some(account_id_hex) = account_id_hex {
            let conn = self.get_conn()?;
            let account_id = AccountID(account_id_hex.to_string());

            transaction(&conn, || {
                if Account::get(&account_id, &conn).is_ok() {
                    let transaction_log = TransactionLog::log_submitted(
                        tx_proposal,
                        block_index,
                        comment.unwrap_or_else(|| "".to_string()),
                        &account_id_hex,
                        &conn,
                    )?;

                    let associated_txos = transaction_log.get_associated_txos(&conn)?;
                    let value_map = transaction_log.value_map(&conn)?;

                    Ok(Some((transaction_log, associated_txos, value_map)))
                } else {
                    Err(TransactionServiceError::Database(
                        WalletDbError::AccountNotFound(account_id_hex),
                    ))
                }
            })
        } else {
            Ok(None)
        }
    }

    fn build_sign_and_submit_transaction(
        &self,
        account_id_hex: &str,
        addresses_and_amounts: &[(String, AmountJSON)],
        input_txo_ids: Option<&Vec<String>>,
        fee_value: Option<String>,
        fee_token_id: Option<String>,
        tombstone_block: Option<String>,
        max_spendable_value: Option<String>,
        comment: Option<String>,
        memo: TransactionMemo,
        block_version: Option<BlockVersion>,
    ) -> Result<(TransactionLog, AssociatedTxos, ValueMap, TxProposal), TransactionServiceError>
    {
        let tx_proposal = self.build_and_sign_transaction(
            account_id_hex,
            addresses_and_amounts,
            input_txo_ids,
            fee_value,
            fee_token_id,
            tombstone_block,
            max_spendable_value,
            memo,
            block_version,
        )?;

        if let Some(transaction_log_and_associated_txos) =
            self.submit_transaction(&tx_proposal, comment, Some(account_id_hex.to_string()))?
        {
            Ok((
                transaction_log_and_associated_txos.0,
                transaction_log_and_associated_txos.1,
                transaction_log_and_associated_txos.2,
                tx_proposal,
            ))
        } else {
            Err(TransactionServiceError::MissingAccountOnSubmit)
        }
    }
}

fn validate_number_inputs(num_inputs: u64) -> Result<(), TransactionServiceError> {
    if num_inputs > MAX_INPUTS {
        return Err(TransactionServiceError::TransactionBuilder(WalletTransactionBuilderError::InvalidArgument(
            format!("Invalid number of input txos. {:?} txo ids provided but maximum allowed number of inputs is {:?}", num_inputs, MAX_INPUTS)
        )));
    }
    Ok(())
}

fn validate_number_outputs(num_outputs: u64) -> Result<(), TransactionServiceError> {
    // maximum number of outputs is 16 but we reserve 1 for change
    let max_outputs = MAX_OUTPUTS - 1;
    if num_outputs > max_outputs {
        return Err(TransactionServiceError::TransactionBuilder(WalletTransactionBuilderError::InvalidArgument(
            format!("Invalid number of recipiants. {:?} recipiants provided but maximum allowed number of outputs is {:?}", num_outputs, max_outputs)
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db::{account::AccountID, models::Txo, txo::TxoModel},
        service::{
            account::AccountService, address::AddressService, balance::BalanceService,
            transaction_log::TransactionLogService,
        },
        test_utils::{
            add_block_to_ledger_db, add_block_with_tx_outs, get_test_ledger, manually_sync_account,
            setup_wallet_service, MOB,
        },
        util::b58::b58_encode_public_address,
    };
    use mc_account_keys::{AccountKey, PublicAddress};
    use mc_common::logger::{test_with_logger, Logger};
    use mc_crypto_keys::RistrettoPublic;
    use mc_crypto_rand::rand_core::RngCore;
    use mc_transaction_core::{
        get_tx_out_shared_secret, ring_signature::KeyImage, tokens::Mob, Token,
    };
    use mc_transaction_extra::AuthenticatedSenderMemo;
    use rand::{rngs::StdRng, SeedableRng};
    use std::convert::TryFrom;

    #[test_with_logger]
    fn test_build_transaction_and_log(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        let service = setup_wallet_service(ledger_db.clone(), logger.clone());

        // Create our main account for the wallet
        let alice = service
            .create_account(
                Some("Alice's Main Account".to_string()),
                "".to_string(),
                "".to_string(),
                "".to_string(),
            )
            .unwrap();

        // Add a block with a transaction for Alice
        let alice_account_key: AccountKey = mc_util_serial::decode(&alice.account_key).unwrap();
        let alice_account_id = AccountID::from(&alice_account_key);
        let alice_public_address = alice_account_key.default_subaddress();

        let tx_logs = service
            .list_transaction_logs(Some(alice_account_id.to_string()), None, None, None, None)
            .unwrap();

        assert_eq!(0, tx_logs.len());

        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![alice_public_address.clone()],
            100 * MOB,
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        manually_sync_account(
            &ledger_db,
            &service.wallet_db.as_ref().unwrap(),
            &alice_account_id,
            &logger,
        );

        let tx_logs = service
            .list_transaction_logs(Some(alice_account_id.to_string()), None, None, None, None)
            .unwrap();

        assert_eq!(0, tx_logs.len());

        // Verify balance for Alice
        let balance = service
            .get_balance_for_account(&AccountID(alice.id.clone()))
            .unwrap();
        let balance_pmob = balance.get(&Mob::ID).unwrap();
        assert_eq!(balance_pmob.unspent, 100 * MOB as u128);

        // Add an account for Bob
        let bob = service
            .create_account(
                Some("Bob's Main Account".to_string()),
                "".to_string(),
                "".to_string(),
                "".to_string(),
            )
            .unwrap();
        let bob_account_key: AccountKey =
            mc_util_serial::decode(&bob.account_key).expect("Could not decode account key");
        let _bob_account_id = AccountID::from(&bob_account_key);

        // Create an assigned subaddress for Bob
        let bob_address_from_alice = service
            .assign_address_for_account(&AccountID(bob.id.clone()), Some("From Alice"))
            .unwrap();

        let _tx_proposal = service
            .build_and_sign_transaction(
                &alice.id,
                &[(
                    bob_address_from_alice.public_address_b58,
                    AmountJSON::new(42 * MOB, Mob::ID),
                )],
                None,
                None,
                None,
                None,
                None,
                TransactionMemo::RTH(None),
                None,
            )
            .unwrap();
        log::info!(logger, "Built transaction from Alice");

        let tx_logs = service
            .list_transaction_logs(Some(alice_account_id.to_string()), None, None, None, None)
            .unwrap();

        assert_eq!(1, tx_logs.len());

        // Create an assigned subaddress for Bob
        let bob_address_from_alice_2 = service
            .assign_address_for_account(&AccountID(bob.id.clone()), Some("From Alice"))
            .unwrap();

        let _tx_proposal = service
            .build_and_sign_transaction(
                &alice.id,
                &[(
                    bob_address_from_alice_2.public_address_b58,
                    AmountJSON::new(42 * MOB, Mob::ID),
                )],
                None,
                None,
                None,
                None,
                None,
                TransactionMemo::RTH(None),
                None,
            )
            .unwrap();
        log::info!(logger, "Built transaction from Alice");

        let tx_logs = service
            .list_transaction_logs(Some(alice_account_id.to_string()), None, None, None, None)
            .unwrap();

        assert_eq!(2, tx_logs.len());

        // Create an assigned subaddress for Bob
        let bob_address_from_alice_3 = service
            .assign_address_for_account(&AccountID(bob.id.clone()), Some("From Alice"))
            .unwrap();

        let _tx_proposal = service
            .build_and_sign_transaction(
                &alice.id,
                &[(
                    bob_address_from_alice_3.clone().public_address_b58,
                    AmountJSON::new(42 * MOB, Mob::ID),
                )],
                None,
                None,
                None,
                None,
                None,
                TransactionMemo::RTH(None),
                None,
            )
            .unwrap();
        log::info!(logger, "Built transaction from Alice");

        let tx_logs = service
            .list_transaction_logs(Some(alice_account_id.to_string()), None, None, None, None)
            .unwrap();

        assert_eq!(3, tx_logs.len());
    }

    // Test sending a transaction from Alice -> Bob, and then from Bob -> Alice
    #[test_with_logger]
    fn test_send_transaction(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        let service = setup_wallet_service(ledger_db.clone(), logger.clone());

        // Create our main account for the wallet
        let alice = service
            .create_account(
                Some("Alice's Main Account".to_string()),
                "".to_string(),
                "".to_string(),
                "".to_string(),
            )
            .unwrap();

        // Add a block with a transaction for Alice
        let alice_account_key: AccountKey = mc_util_serial::decode(&alice.account_key).unwrap();
        let alice_account_id = AccountID::from(&alice_account_key);
        let alice_public_address = alice_account_key.default_subaddress();
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![alice_public_address.clone()],
            100 * MOB,
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        manually_sync_account(
            &ledger_db,
            &service.wallet_db.as_ref().unwrap(),
            &alice_account_id,
            &logger,
        );

        // Verify balance for Alice
        let balance = service
            .get_balance_for_account(&AccountID(alice.id.clone()))
            .unwrap();
        let balance_pmob = balance.get(&Mob::ID).unwrap();
        assert_eq!(balance_pmob.unspent, 100 * MOB as u128);

        // Add an account for Bob
        let bob = service
            .create_account(
                Some("Bob's Main Account".to_string()),
                "".to_string(),
                "".to_string(),
                "".to_string(),
            )
            .unwrap();
        let bob_account_key: AccountKey =
            mc_util_serial::decode(&bob.account_key).expect("Could not decode account key");
        let bob_account_id = AccountID::from(&bob_account_key);

        // Create an assigned subaddress for Bob
        let bob_address_from_alice = service
            .assign_address_for_account(&AccountID(bob.id.clone()), Some("From Alice"))
            .unwrap();

        // Send a transaction from Alice to Bob
        let (transaction_log, _associated_txos, _value_map, tx_proposal) = service
            .build_sign_and_submit_transaction(
                &alice.id,
                &[(
                    bob_address_from_alice.public_address_b58,
                    AmountJSON::new(42 * MOB, Mob::ID),
                )],
                None,
                None,
                None,
                None,
                None,
                None,
                TransactionMemo::RTH(None),
                None,
            )
            .unwrap();
        log::info!(logger, "Built and submitted transaction from Alice");

        // NOTE: Submitting to the test ledger via propose_tx doesn't actually add the
        // block to the ledger, because no consensus is occurring, so this is the
        // workaround.
        {
            log::info!(logger, "Adding block from transaction log");
            let key_images: Vec<KeyImage> = tx_proposal
                .input_txos
                .iter()
                .map(|txo| txo.key_image.clone())
                .collect();

            // Note: This block doesn't contain the fee output.
            add_block_with_tx_outs(
                &mut ledger_db,
                &[
                    tx_proposal.change_txos[0].tx_out.clone(),
                    tx_proposal.payload_txos[0].tx_out.clone(),
                ],
                &key_images,
                &mut rng,
            );
        }

        manually_sync_account(
            &ledger_db,
            &service.wallet_db.as_ref().unwrap(),
            &alice_account_id,
            &logger,
        );
        manually_sync_account(
            &ledger_db,
            &service.wallet_db.as_ref().unwrap(),
            &bob_account_id,
            &logger,
        );

        // Get the Txos from the transaction log
        let transaction_txos = transaction_log
            .get_associated_txos(&service.get_conn().unwrap())
            .unwrap();
        let secreted = transaction_txos
            .outputs
            .iter()
            .map(|(t, _)| Txo::get(&t.id, &service.get_conn().unwrap()).unwrap())
            .collect::<Vec<Txo>>();
        assert_eq!(secreted.len(), 1);
        assert_eq!(secreted[0].value as u64, 42 * MOB);

        let change = transaction_txos
            .change
            .iter()
            .map(|(t, _)| Txo::get(&t.id, &service.get_conn().unwrap()).unwrap())
            .collect::<Vec<Txo>>();
        assert_eq!(change.len(), 1);
        assert_eq!(change[0].value as u64, 58 * MOB - Mob::MINIMUM_FEE);

        let inputs = transaction_txos
            .inputs
            .iter()
            .map(|t| Txo::get(&t.id, &service.get_conn().unwrap()).unwrap())
            .collect::<Vec<Txo>>();
        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0].value as u64, 100 * MOB);

        // Verify balance for Alice = original balance - fee - txo_value
        let balance = service
            .get_balance_for_account(&AccountID(alice.id.clone()))
            .unwrap();
        let balance_pmob = balance.get(&Mob::ID).unwrap();
        assert_eq!(balance_pmob.unspent, (58 * MOB - Mob::MINIMUM_FEE) as u128);

        // Bob's balance should be = output_txo_value
        let bob_balance = service
            .get_balance_for_account(&AccountID(bob.id.clone()))
            .unwrap();
        let bob_balance_pmob = bob_balance.get(&Mob::ID).unwrap();
        assert_eq!(bob_balance_pmob.unspent, 42000000000000);

        // Bob should now be able to send to Alice
        let (_, _, _, tx_proposal) = service
            .build_sign_and_submit_transaction(
                &bob.id,
                &[(
                    b58_encode_public_address(&alice_public_address).unwrap(),
                    AmountJSON::new(8 * MOB, Mob::ID),
                )],
                None,
                None,
                None,
                None,
                None,
                None,
                TransactionMemo::RTH(None),
                None,
            )
            .unwrap();

        // NOTE: Submitting to the test ledger via propose_tx doesn't actually add the
        // block to the ledger, because no consensus is occurring, so this is the
        // workaround.

        {
            log::info!(logger, "Adding block from transaction proposal");
            let key_images: Vec<KeyImage> = tx_proposal
                .input_txos
                .iter()
                .map(|txo| txo.key_image.clone())
                .collect();

            // Note: This block doesn't contain the fee output.
            add_block_with_tx_outs(
                &mut ledger_db,
                &[
                    tx_proposal.change_txos[0].tx_out.clone(),
                    tx_proposal.payload_txos[0].tx_out.clone(),
                ],
                &key_images,
                &mut rng,
            );
        }

        manually_sync_account(
            &ledger_db,
            &service.wallet_db.as_ref().unwrap(),
            &alice_account_id,
            &logger,
        );
        manually_sync_account(
            &ledger_db,
            &service.wallet_db.as_ref().unwrap(),
            &bob_account_id,
            &logger,
        );

        let alice_balance = service
            .get_balance_for_account(&AccountID(alice.id))
            .unwrap();
        let alice_balance_pmob = alice_balance.get(&Mob::ID).unwrap();
        assert_eq!(
            alice_balance_pmob.unspent,
            (66 * MOB - Mob::MINIMUM_FEE) as u128
        );

        // Bob's balance should be = output_txo_value
        let bob_balance = service.get_balance_for_account(&AccountID(bob.id)).unwrap();
        let bob_balance_pmob = bob_balance.get(&Mob::ID).unwrap();
        assert_eq!(
            bob_balance_pmob.unspent,
            (34 * MOB - Mob::MINIMUM_FEE) as u128
        );
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
            .create_account(
                Some("Alice's Main Account".to_string()),
                "".to_string(),
                "".to_string(),
                "".to_string(),
            )
            .unwrap();

        // Add a block with a transaction for Alice
        let alice_account_key: AccountKey = mc_util_serial::decode(&alice.account_key).unwrap();
        let alice_account_id = AccountID::from(&alice_account_key);
        let alice_public_address = alice_account_key.default_subaddress();
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![alice_public_address.clone()],
            100 * MOB,
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        manually_sync_account(
            &ledger_db,
            &service.wallet_db.as_ref().unwrap(),
            &alice_account_id,
            &logger,
        );

        match service.build_and_sign_transaction(
            &alice.id,
            &vec![("NOTB58".to_string(), AmountJSON::new(42 * MOB, Mob::ID))],
            None,
            None,
            None,
            None,
            None,
            TransactionMemo::RTH(None),
            None,
        ) {
            Ok(_) => {
                panic!("Should not be able to build transaction to invalid b58 public address")
            }
            Err(TransactionServiceError::InvalidPublicAddress(_)) => {}
            Err(e) => panic!("Unexpected error {:?}", e),
        };
    }

    #[test_with_logger]
    fn test_maximum_inputs_and_outputs(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        let service = setup_wallet_service(ledger_db.clone(), logger.clone());

        // Create our main account for the wallet
        let alice = service
            .create_account(
                Some("Alice's Main Account".to_string()),
                "".to_string(),
                "".to_string(),
                "".to_string(),
            )
            .unwrap();

        // Add a block with a transaction for Alice
        let alice_account_key: AccountKey = mc_util_serial::decode(&alice.account_key).unwrap();
        let alice_account_id = AccountID::from(&alice_account_key);
        let alice_public_address = alice_account_key.default_subaddress();
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![alice_public_address.clone()],
            100 * MOB,
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        manually_sync_account(
            &ledger_db,
            &service.wallet_db.as_ref().unwrap(),
            &alice_account_id,
            &logger,
        );

        // test ouputs
        let mut outputs = Vec::new();
        for _ in 0..17 {
            outputs.push((
                b58_encode_public_address(&alice_public_address).unwrap(),
                AmountJSON::new(42 * MOB, Mob::ID),
            ));
        }
        match service.build_and_sign_transaction(
            &alice.id,
            &outputs,
            None,
            None,
            None,
            None,
            None,
            TransactionMemo::RTH(None),
            None,
        ) {
            Ok(_) => {
                panic!("Should not be able to build transaction with too many ouputs")
            }
            Err(TransactionServiceError::TransactionBuilder(
                WalletTransactionBuilderError::InvalidArgument(_),
            )) => {}
            Err(e) => panic!("Unexpected error {:?}", e),
        };

        // test inputs
        let mut outputs = Vec::new();
        for _ in 0..2 {
            outputs.push((
                b58_encode_public_address(&alice_public_address).unwrap(),
                AmountJSON::new(42 * MOB, Mob::ID),
            ));
        }
        let mut inputs = Vec::new();
        for _ in 0..17 {
            inputs.push("fake txo id".to_string());
        }
        match service.build_and_sign_transaction(
            &alice.id,
            &outputs,
            Some(&inputs),
            None,
            None,
            None,
            None,
            TransactionMemo::RTH(None),
            None,
        ) {
            Ok(_) => {
                panic!("Should not be able to build transaction with too many inputs")
            }
            Err(TransactionServiceError::TransactionBuilder(
                WalletTransactionBuilderError::InvalidArgument(_),
            )) => {}
            Err(e) => panic!("Unexpected error {:?}", e),
        };
    }

    // Test sending a transaction from Alice -> Bob, and then from Bob -> Alice
    #[test_with_logger]
    fn test_send_transaction_with_sender_memo_cred_subaddress_index(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        let service = setup_wallet_service(ledger_db.clone(), logger.clone());

        // Create our main account for the wallet
        let alice = service
            .create_account(
                Some("Alice's Main Account".to_string()),
                "".to_string(),
                "".to_string(),
                "".to_string(),
            )
            .unwrap();

        // Add a block with a transaction for Alice
        let alice_account_key: AccountKey = mc_util_serial::decode(&alice.account_key).unwrap();
        let alice_account_id = AccountID::from(&alice_account_key);
        let alice_public_address = alice_account_key.default_subaddress();
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![alice_public_address.clone()],
            100 * MOB,
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        manually_sync_account(
            &ledger_db,
            &service.wallet_db.as_ref().unwrap(),
            &alice_account_id,
            &logger,
        );

        // Verify balance for Alice
        let balance = service
            .get_balance_for_account(&AccountID(alice.id.clone()))
            .unwrap();
        let balance_pmob = balance.get(&Mob::ID).unwrap();
        assert_eq!(balance_pmob.unspent, 100 * MOB as u128);

        // Add an account for Bob
        let bob = service
            .create_account(
                Some("Bob's Main Account".to_string()),
                "".to_string(),
                "".to_string(),
                "".to_string(),
            )
            .unwrap();
        let bob_account_key: AccountKey =
            mc_util_serial::decode(&bob.account_key).expect("Could not decode account key");
        let bob_account_id = AccountID::from(&bob_account_key);

        // Create an assigned subaddress for Bob to receive funds from Alice
        let bob_address_from_alice = service
            .assign_address_for_account(&AccountID(bob.id.clone()), Some("From Alice"))
            .unwrap();

        // Create an assigned subaddress for Alice to receive from Bob, which will be
        // used to authenticate the sender (Alice)
        let alice_address_from_bob = service
            .assign_address_for_account(&alice_account_id, Some("From Bob"))
            .unwrap();

        // Send a transaction from Alice to Bob
        let (transaction_log, _associated_txos, _value_map, tx_proposal) = service
            .build_sign_and_submit_transaction(
                &alice.id,
                &[(
                    bob_address_from_alice.public_address_b58,
                    AmountJSON::new(42 * MOB, Mob::ID),
                )],
                None,
                None,
                None,
                None,
                None,
                None,
                TransactionMemo::RTH(Some(alice_address_from_bob.subaddress_index as u64)),
                None,
            )
            .unwrap();
        log::info!(logger, "Built and submitted transaction from Alice");

        // NOTE: Submitting to the test ledger via propose_tx doesn't actually add the
        // block to the ledger, because no consensus is occurring, so this is the
        // workaround.
        {
            log::info!(logger, "Adding block from transaction log");
            let key_images: Vec<KeyImage> = tx_proposal
                .input_txos
                .iter()
                .map(|txo| txo.key_image.clone())
                .collect();

            // Note: This block doesn't contain the fee output.
            add_block_with_tx_outs(
                &mut ledger_db,
                &[
                    tx_proposal.change_txos[0].tx_out.clone(),
                    tx_proposal.payload_txos[0].tx_out.clone(),
                ],
                &key_images,
                &mut rng,
            );
        }

        manually_sync_account(
            &ledger_db,
            &service.wallet_db.as_ref().unwrap(),
            &alice_account_id,
            &logger,
        );
        manually_sync_account(
            &ledger_db,
            &service.wallet_db.as_ref().unwrap(),
            &bob_account_id,
            &logger,
        );

        // Get the Txos from the transaction log
        let transaction_txos = transaction_log
            .get_associated_txos(&service.get_conn().unwrap())
            .unwrap();
        let secreted = transaction_txos
            .outputs
            .iter()
            .map(|(t, _)| Txo::get(&t.id, &service.get_conn().unwrap()).unwrap())
            .collect::<Vec<Txo>>();
        assert_eq!(secreted.len(), 1);
        assert_eq!(secreted[0].value as u64, 42 * MOB);

        let change = transaction_txos
            .change
            .iter()
            .map(|(t, _)| Txo::get(&t.id, &service.get_conn().unwrap()).unwrap())
            .collect::<Vec<Txo>>();
        assert_eq!(change.len(), 1);
        assert_eq!(change[0].value as u64, 58 * MOB - Mob::MINIMUM_FEE);

        let inputs = transaction_txos
            .inputs
            .iter()
            .map(|t| Txo::get(&t.id, &service.get_conn().unwrap()).unwrap())
            .collect::<Vec<Txo>>();
        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0].value as u64, 100 * MOB);

        // Verify balance for Alice = original balance - fee - txo_value
        let balance = service
            .get_balance_for_account(&AccountID(alice.id.clone()))
            .unwrap();
        let balance_pmob = balance.get(&Mob::ID).unwrap();
        assert_eq!(balance_pmob.unspent, (58 * MOB - Mob::MINIMUM_FEE) as u128);

        // Bob's balance should be = output_txo_value
        let bob_balance = service
            .get_balance_for_account(&AccountID(bob.id.clone()))
            .unwrap();
        let bob_balance_pmob = bob_balance.get(&Mob::ID).unwrap();
        assert_eq!(bob_balance_pmob.unspent, 42000000000000);

        // Decrypt the memo from the transaction txo and verify.
        let txo = &tx_proposal.payload_txos[0].tx_out;

        let shared_secret = get_tx_out_shared_secret(
            bob_account_key.view_private_key(),
            &RistrettoPublic::try_from(&txo.public_key).unwrap(),
        );

        let memo = txo.decrypt_memo(&shared_secret);
        let authenticated_sender_memo = AuthenticatedSenderMemo::from(memo.get_memo_data());
        let validation = authenticated_sender_memo.validate(
            &alice_account_key.subaddress(alice_address_from_bob.subaddress_index as u64),
            &bob_account_key
                .subaddress_view_private(bob_address_from_alice.subaddress_index as u64),
            &txo.public_key,
        );

        assert!(bool::from(validation));
    }

    // FIXME: Test with 0 change transactions
    // FIXME: Test with balance > u64::max
    // FIXME: sending a transaction with value > u64::max
}
