// Copyright (c) 2020-2021 MobileCoin Inc.

//! Service for managing gift codes.
//!
//! Gift codes are onetime accounts that contain a single Txo. They provide
//! a means to send MOB in a way that can be "claimed," for example, by pasting
//! a QR code for a gift code into a group chat, and the first person to
//! consume the gift code claims the MOB.

use crate::{
    db::{
        account::{AccountID, AccountModel},
        assigned_subaddress::AssignedSubaddressModel,
        b58_encode,
        gift_code::GiftCodeModel,
        models::{Account, AssignedSubaddress, GiftCode, TransactionLog, Txo},
        txo::{TxoID, TxoModel},
        WalletDbError,
    },
    service::{
        account::{AccountService, AccountServiceError},
        address::AddressService,
        transaction::{TransactionService, TransactionServiceError},
        WalletService,
    },
};
use diesel::prelude::*;
use displaydoc::Display;
use mc_account_keys::{AccountKey, RootEntropy, RootIdentity, DEFAULT_SUBADDRESS_INDEX};
use mc_common::logger::log;
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_crypto_keys::{CompressedRistrettoPublic, RistrettoPublic};
use mc_fog_report_validation::FogPubkeyResolver;
use mc_ledger_db::Ledger;
use mc_mobilecoind::payments::TxProposal;
use mc_transaction_core::{
    constants::MINIMUM_FEE, get_tx_out_shared_secret, onetime_keys::recover_onetime_private_key,
    ring_signature::KeyImage,
};
use mc_util_from_random::FromRandom;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt};

#[derive(Display, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum GiftCodeServiceError {
    /// Error interacting with the database: {0}
    Database(WalletDbError),

    /// Error with LedgerDB: {0}
    LedgerDB(mc_ledger_db::Error),

    /// Error decoding from hex: {0}
    HexDecode(hex::FromHexError),

    /// Error decoding prost: {0}
    ProstDecode(prost::DecodeError),

    /// Building the gift code failed
    BuildGiftCodeFailed,

    /// Unexpected TxStatus while polling: {0}
    UnexpectedTxStatus(String),

    /// Gift Code transaction produced an unexpected number of outputs: {0}
    UnexpectedNumOutputs(usize),

    /// Gift Code does not contain enough value to cover the fee: {0}
    InsufficientValueForFee(u64),

    /// Unexpected number of Txos in the Gift Code Account: {0}
    UnexpectedNumTxosInGiftCodeAccount(usize),

    /// Unexpected Value in Gift Code Txo: {0}
    UnexpectedValueInGiftCodeTxo(u64),

    /// The Txo is not consumable
    TxoNotConsumable,

    /// The TxProposal for this GiftCode was constructed in an unexpected
    /// manner.
    UnexpectedTxProposalFormat,

    /// Diesel error: {0}
    Diesel(diesel::result::Error),

    /// Error with the Transaction Service: {0}
    TransactionService(TransactionServiceError),

    /// Error with the Account Service: {0}
    AccountService(AccountServiceError),

    /// Error with printable wrapper: {0}
    PrintableWrapper(mc_api::display::Error),

    /// Error with crypto keys: {0}
    CryptoKey(mc_crypto_keys::KeyError),

    /// Gift Code Txo is not in ledger at block index: {0}
    GiftCodeTxoNotInLedger(u64),

    /// Cannot claim a gift code that has already been claimed
    GiftCodeClaimed,

    /// Cannot claim a gift code which has not yet landed in the ledger
    GiftCodeNotYetAvailable,
}

impl From<WalletDbError> for GiftCodeServiceError {
    fn from(src: WalletDbError) -> Self {
        Self::Database(src)
    }
}

impl From<mc_ledger_db::Error> for GiftCodeServiceError {
    fn from(src: mc_ledger_db::Error) -> Self {
        Self::LedgerDB(src)
    }
}

impl From<hex::FromHexError> for GiftCodeServiceError {
    fn from(src: hex::FromHexError) -> Self {
        Self::HexDecode(src)
    }
}

impl From<prost::DecodeError> for GiftCodeServiceError {
    fn from(src: prost::DecodeError) -> Self {
        Self::ProstDecode(src)
    }
}

impl From<diesel::result::Error> for GiftCodeServiceError {
    fn from(src: diesel::result::Error) -> Self {
        Self::Diesel(src)
    }
}

impl From<TransactionServiceError> for GiftCodeServiceError {
    fn from(src: TransactionServiceError) -> Self {
        Self::TransactionService(src)
    }
}

impl From<AccountServiceError> for GiftCodeServiceError {
    fn from(src: AccountServiceError) -> Self {
        Self::AccountService(src)
    }
}

impl From<mc_api::display::Error> for GiftCodeServiceError {
    fn from(src: mc_api::display::Error) -> Self {
        Self::PrintableWrapper(src)
    }
}

impl From<mc_crypto_keys::KeyError> for GiftCodeServiceError {
    fn from(src: mc_crypto_keys::KeyError) -> Self {
        Self::CryptoKey(src)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct EncodedGiftCode(pub String);

impl fmt::Display for EncodedGiftCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The decoded details from the Gift Code.
pub struct DecodedGiftCode {
    root_entropy: RootEntropy,
    txo_public_key: CompressedRistrettoPublic,
    memo: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct GiftCodeEntropy(pub String);

impl fmt::Display for GiftCodeEntropy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Possible states for a Gift Code in relation to accounts in this wallet.
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum GiftCodeStatus {
    /// The Gift Code has been submitted, but has not yet hit the ledger.
    GiftCodeSubmittedPending,

    /// The Gift Code Txo is in the ledger and has not yet been claimed.
    GiftCodeAvailable,

    /// The Gift Code Txo has been spent.
    GiftCodeClaimed,
}

/// Trait defining the ways in which the wallet can interact with and manage
/// gift codes.
pub trait GiftCodeService {
    /// Builds a new gift code.
    ///
    /// Building a gift code requires the following steps:
    ///  1. Create a new account to receive the funds
    ///  2. Send a transaction to the new account
    ///  3. Wait for the transaction to land
    ///  4. Package the required information into a b58-encoded string
    ///
    /// Returns:
    /// * JsonSubmitResponse from submitting the gift code transaction to the
    ///   network
    /// * Entropy of the gift code account, hex encoded
    #[allow(clippy::too_many_arguments)]
    fn build_gift_code(
        &self,
        from_account_id: &AccountID,
        value: u64,
        name: Option<String>,
        input_txo_ids: Option<&Vec<String>>,
        fee: Option<u64>,
        tombstone_block: Option<u64>,
        max_spendable_value: Option<u64>,
    ) -> Result<(TxProposal, EncodedGiftCode, GiftCode), GiftCodeServiceError>;

    /// Get the details for a specific gift code.
    fn get_gift_code(
        &self,
        gift_code_b58: &EncodedGiftCode,
    ) -> Result<GiftCode, GiftCodeServiceError>;

    /// List all gift codes in the wallet.
    fn list_gift_codes(&self) -> Result<Vec<GiftCode>, GiftCodeServiceError>;

    /// Check the status of a gift code currently in your wallet.
    fn check_gift_code_status(
        &self,
        gift_code_b58: &EncodedGiftCode,
    ) -> Result<GiftCodeStatus, GiftCodeServiceError>;

    /// Execute a transaction from the gift code account to drain the account to
    /// the destination specified by the account_id_hex and
    /// assigned_subaddress_b58. If no assigned_subaddress_b58 is provided,
    /// then a new AssignedSubaddress will be created to receive the funds.
    fn claim_gift_code(
        &self,
        gift_code_b58: &EncodedGiftCode,
        account_id: &AccountID,
        assigned_subaddress_b58: Option<String>,
    ) -> Result<(TransactionLog, GiftCode), GiftCodeServiceError>;

    /// Decode the gift code from b58 to its component parts.
    fn decode_gift_code(
        &self,
        gift_code_b58: &EncodedGiftCode,
    ) -> Result<DecodedGiftCode, GiftCodeServiceError>;
}

impl<T, FPR> GiftCodeService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn build_gift_code(
        &self,
        from_account_id: &AccountID,
        value: u64,
        memo: Option<String>,
        input_txo_ids: Option<&Vec<String>>,
        fee: Option<u64>,
        tombstone_block: Option<u64>,
        max_spendable_value: Option<u64>,
    ) -> Result<(TxProposal, EncodedGiftCode, GiftCode), GiftCodeServiceError> {
        // First, create the account which will receive the funds, from a new random
        // entropy
        let mut rng = rand::thread_rng();
        let root_id = RootIdentity::from_random(&mut rng);
        let entropy_str = hex::encode(&root_id.root_entropy);

        // Set first_block to current block height, since we know this account has only
        // existed since now
        let block_index = self.ledger_db.num_blocks()? - 1;
        log::debug!(
            self.logger,
            "Created gift code account. Importing to wallet at block index {:?}.",
            block_index,
        );
        let account = self.import_account(
            entropy_str,
            memo.clone(),
            Some(block_index),
            None,
            None,
            None,
        )?;

        let (gift_code_account, gift_code_account_key, from_account) = {
            let conn = self.wallet_db.get_conn()?;
            // Send a transaction to the gift_code account
            let (gift_code_account, gift_code_account_key, from_account) =
                conn.transaction::<(Account, AccountKey, Account), GiftCodeServiceError, _>(
                    || {
                        let from_account = Account::get(&from_account_id, &conn)?;
                        let gift_code_account =
                            Account::get(&AccountID(account.account_id_hex), &conn)?;

                        let gift_code_account_key: AccountKey =
                            mc_util_serial::decode(&gift_code_account.account_key)?;
                        log::debug!(
                            self.logger,
                            "Funding gift code account {:?} from account {:?}",
                            gift_code_account.account_id_hex,
                            from_account.account_id_hex,
                        );
                        Ok((gift_code_account, gift_code_account_key, from_account))
                    },
                )?;
            (gift_code_account, gift_code_account_key, from_account)
        };

        let main_subaddress =
            gift_code_account_key.subaddress(gift_code_account.main_subaddress_index as u64);
        let gift_code_address = b58_encode(&main_subaddress)?;

        let tx_proposal = self.build_transaction(
            &from_account.account_id_hex,
            &gift_code_address,
            value.to_string(),
            input_txo_ids,
            fee.map(|f| f.to_string()),
            tombstone_block.map(|t| t.to_string()),
            max_spendable_value.map(|f| f.to_string()),
        )?;

        // Create the gift_code_b58 using the printable wrapper for a TransferPayload.
        if tx_proposal.outlay_index_to_tx_out_index.len() != 1 {
            return Err(GiftCodeServiceError::UnexpectedTxProposalFormat);
        }
        let outlay_index = tx_proposal.outlay_index_to_tx_out_index[&0];
        let value = tx_proposal.outlays[0].value;
        let tx_out = tx_proposal.tx.prefix.outputs[outlay_index].clone();
        let txo_public_key = tx_out.public_key;
        let proto_tx_pubkey: mc_api::external::CompressedRistretto = (&txo_public_key).into();

        let mut gift_code_payload = mc_mobilecoind_api::printable::TransferPayload::new();
        gift_code_payload.set_entropy(root_id.root_entropy.bytes.to_vec());
        gift_code_payload.set_tx_out_public_key(proto_tx_pubkey);
        gift_code_payload.set_memo(memo.clone().unwrap_or_else(|| "".to_string()));

        let mut gift_code_wrapper = mc_mobilecoind_api::printable::PrintableWrapper::new();
        gift_code_wrapper.set_transfer_payload(gift_code_payload);
        let gift_code_b58 = gift_code_wrapper.b58_encode()?;

        // Add the gift code to our Gift Codes table for tracking.
        let gift_code = GiftCode::create(
            &EncodedGiftCode(gift_code_b58.clone()),
            &root_id.root_entropy,
            &txo_public_key,
            value as i64,
            memo.unwrap_or_else(|| "".to_string()),
            &AccountID::from(&gift_code_account_key),
            &TxoID::from(&tx_out),
            &self.wallet_db.get_conn()?,
        )?;

        Ok((tx_proposal, EncodedGiftCode(gift_code_b58), gift_code))
    }

    fn get_gift_code(
        &self,
        gift_code_b58: &EncodedGiftCode,
    ) -> Result<GiftCode, GiftCodeServiceError> {
        let conn = self.wallet_db.get_conn()?;
        Ok(GiftCode::get(&gift_code_b58, &conn)?)
    }

    fn list_gift_codes(&self) -> Result<Vec<GiftCode>, GiftCodeServiceError> {
        let conn = self.wallet_db.get_conn()?;
        Ok(GiftCode::list_all(&conn)?)
    }

    fn check_gift_code_status(
        &self,
        gift_code_b58: &EncodedGiftCode,
    ) -> Result<GiftCodeStatus, GiftCodeServiceError> {
        let decoded_gift_code = self.decode_gift_code(gift_code_b58)?;
        let account_key = AccountKey::from(&RootIdentity::from(&decoded_gift_code.root_entropy));

        // Check if the GiftCode is in the local ledger.
        let _gift_txo = match self
            .ledger_db
            .get_tx_out_index_by_public_key(&decoded_gift_code.txo_public_key)
        {
            Ok(tx_out_index) => self.ledger_db.get_tx_out_by_index(tx_out_index)?,
            Err(mc_ledger_db::Error::NotFound) => {
                return Ok(GiftCodeStatus::GiftCodeSubmittedPending)
            }
            Err(e) => return Err(e.into()),
        };

        // Check if the Gift Code has been spent - by convention gift codes are always
        // to the main subaddress index.
        let gift_code_subaddress = DEFAULT_SUBADDRESS_INDEX;
        let gift_code_key_image = {
            let onetime_private_key = recover_onetime_private_key(
                &RistrettoPublic::try_from(&decoded_gift_code.txo_public_key)?,
                account_key.view_private_key(),
                &account_key.subaddress_spend_private(gift_code_subaddress as u64),
            );
            KeyImage::from(&onetime_private_key)
        };

        if self.ledger_db.contains_key_image(&gift_code_key_image)? {
            return Ok(GiftCodeStatus::GiftCodeClaimed);
        }

        Ok(GiftCodeStatus::GiftCodeAvailable)
    }

    fn claim_gift_code(
        &self,
        gift_code_b58: &EncodedGiftCode,
        account_id: &AccountID,
        assigned_subaddress_b58: Option<String>,
    ) -> Result<(TransactionLog, GiftCode), GiftCodeServiceError> {
        log::info!(
            self.logger,
            "Consuming gift code {:?} to account_id {:?} at address {:?}",
            gift_code_b58,
            account_id,
            assigned_subaddress_b58
        );

        match self.check_gift_code_status(gift_code_b58)? {
            GiftCodeStatus::GiftCodeClaimed => return Err(GiftCodeServiceError::GiftCodeClaimed),
            GiftCodeStatus::GiftCodeSubmittedPending => {
                return Err(GiftCodeServiceError::GiftCodeNotYetAvailable)
            }
            GiftCodeStatus::GiftCodeAvailable => {}
        }

        // Get the components of the gift code from the printable wrapper
        let decoded = self.decode_gift_code(gift_code_b58)?;

        // Get the block height to start scanning based on the block index of the
        // tx_out_public_key
        let tx_out_index = self
            .ledger_db
            .get_tx_out_index_by_public_key(&decoded.txo_public_key)?;
        let scan_block = self
            .ledger_db
            .get_block_index_by_tx_out_index(tx_out_index)?;

        // Get the value of the txo in the gift
        let gift_txo = self.ledger_db.get_tx_out_by_index(tx_out_index)?;
        let root_id = RootIdentity::from(&decoded.root_entropy);
        let gift_code_account_key = AccountKey::from(&root_id);
        let shared_secret = get_tx_out_shared_secret(
            gift_code_account_key.view_private_key(),
            &RistrettoPublic::try_from(&gift_txo.public_key).unwrap(),
        );
        let (value, _blinding) = gift_txo.amount.get_value(&shared_secret).unwrap();

        // Add this account to our DB. It will be drained immediately.
        let gift_code_account_id_hex = match Account::get(
            &AccountID::from(&gift_code_account_key),
            &self.wallet_db.get_conn()?,
        ) {
            // The account may already be in the wallet if we constructed this gift code in this
            // wallet.
            Ok(account) => account.account_id_hex,
            Err(WalletDbError::AccountNotFound(_)) => {
                let account = self.import_account(
                    hex::encode(decoded.root_entropy.bytes),
                    Some(format!("Gift Code: {}", decoded.memo)),
                    Some(scan_block),
                    None,
                    None,
                    None,
                )?;
                log::info!(
                    self.logger,
                    "Imported gift code account {:?}.",
                    account.account_id_hex
                );
                account.account_id_hex
            }
            Err(e) => return Err(e.into()),
        };

        // Construct a transaction from the gift code account to our desired recipient
        // account.
        let destination_address = assigned_subaddress_b58.unwrap_or_else(|| {
            let address = self
                .assign_address_for_account(
                    &account_id,
                    Some(&format!("Gift Code: {}", decoded.memo)),
                )
                .unwrap();
            address.assigned_subaddress_b58
        });
        log::info!(
            self.logger,
            "Consuming gift code to destination address {:?}",
            destination_address
        );

        if value < MINIMUM_FEE {
            return Err(GiftCodeServiceError::InsufficientValueForFee(value));
        }

        // Sanity check that we have assigned subaddresses for the gift code account
        let addresses =
            AssignedSubaddress::list_all(&gift_code_account_id_hex, &self.wallet_db.get_conn()?)?;
        assert_eq!(addresses.len(), 2);

        // Sanity check that our txo is available and spendable from the gift code
        // account
        let txos = Txo::list_for_account(&gift_code_account_id_hex, &self.wallet_db.get_conn()?)?;
        if txos.is_empty() {
            return Err(GiftCodeServiceError::GiftCodeTxoNotInLedger(
                self.ledger_db.num_blocks()? - 1,
            ));
        }
        if txos.len() != 1 {
            return Err(GiftCodeServiceError::UnexpectedNumTxosInGiftCodeAccount(
                txos.len(),
            ));
        }
        if txos[0].txo.value as u64 != value {
            return Err(GiftCodeServiceError::UnexpectedValueInGiftCodeTxo(
                txos[0].txo.value as u64,
            ));
        }
        let mut txo = txos[0].clone();
        let max_polling = 3;
        let mut count = 0;
        while txo.txo.subaddress_index.is_none() && count <= max_polling {
            if count == max_polling {
                return Err(GiftCodeServiceError::TxoNotConsumable);
            }
            // Note that we now need to allow the sync thread to catch up for this TXO so
            // that we can make sure the subaddress is assigned, rendering the
            // Txo spendable.
            std::thread::sleep(std::time::Duration::from_secs(3));
            let txos =
                Txo::list_for_account(&gift_code_account_id_hex, &self.wallet_db.get_conn()?)?;
            txo = txos[0].clone();
            count += 1;
        }

        // We go with all the defaults because there is only one TXO in this account to
        // spend.
        let (transaction_log, _associated_txos) = self.build_and_submit(
            &gift_code_account_id_hex,
            &destination_address,
            (value - MINIMUM_FEE).to_string(),
            None,
            Some(MINIMUM_FEE.to_string()),
            None,
            None,
            Some(
                json!({ "claim_gift_code": decoded.memo, "recipient_address": destination_address })
                    .to_string(),
            ),
        )?;
        log::info!(
            self.logger,
            "Submitted transaction to consume gift code with id {:?}",
            transaction_log.transaction_id_hex
        );

        let gift_code = GiftCode::create(
            gift_code_b58,
            &decoded.root_entropy,
            &decoded.txo_public_key,
            value as i64,
            decoded.memo.clone(),
            &AccountID(gift_code_account_id_hex),
            &TxoID(txo.txo.txo_id_hex),
            &self.wallet_db.get_conn()?,
        )?;
        Ok((transaction_log, gift_code))
    }

    fn decode_gift_code(
        &self,
        gift_code_b58: &EncodedGiftCode,
    ) -> Result<DecodedGiftCode, GiftCodeServiceError> {
        let wrapper =
            mc_mobilecoind_api::printable::PrintableWrapper::b58_decode(gift_code_b58.to_string())
                .unwrap();
        let transfer_payload = wrapper.get_transfer_payload();

        let mut entropy = [0u8; 32];
        entropy.copy_from_slice(transfer_payload.get_entropy());
        let root_entropy = RootEntropy::from(&entropy);

        let txo_public_key =
            CompressedRistrettoPublic::try_from(transfer_payload.get_tx_out_public_key()).unwrap();

        Ok(DecodedGiftCode {
            root_entropy,
            txo_public_key,
            memo: transfer_payload.get_memo().to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db::{b58_decode, transaction_log::TransactionLogModel},
        service::balance::BalanceService,
        test_utils::{
            add_block_from_transaction_log, add_block_to_ledger_db, add_block_with_tx_proposal,
            get_test_ledger, manually_sync_account, setup_wallet_service, MOB,
        },
    };
    use mc_account_keys::PublicAddress;
    use mc_common::logger::{test_with_logger, Logger};
    use mc_crypto_rand::rand_core::RngCore;
    use mc_transaction_core::ring_signature::KeyImage;
    use rand::{rngs::StdRng, SeedableRng};

    #[test_with_logger]
    fn test_gift_code_lifecycle(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        let service = setup_wallet_service(ledger_db.clone(), logger.clone());

        // Create our main account for the wallet
        let alice = service
            .create_account(Some("Alice's Main Account".to_string()), None)
            .unwrap();

        // Add a block with a transaction for Alice
        let alice_account_key: AccountKey = mc_util_serial::decode(&alice.account_key).unwrap();
        let alice_public_address =
            &alice_account_key.subaddress(alice.main_subaddress_index as u64);
        let alice_account_id = AccountID(alice.account_id_hex.to_string());

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
            &alice_account_id,
            13,
            &logger,
        );

        // Verify balance for Alice
        let balance = service
            .get_balance_for_account(&AccountID(alice.account_id_hex.clone()))
            .unwrap();
        assert_eq!(balance.unspent, 100 * MOB as u64);

        // Create a gift code for Bob
        let (tx_proposal, gift_code_b58, _db_gift_code) = service
            .build_gift_code(
                &AccountID(alice.account_id_hex.clone()),
                2 * MOB as u64,
                Some("Gift code for Bob".to_string()),
                None,
                None,
                None,
                None,
            )
            .unwrap();
        log::info!(logger, "Built and submitted gift code transaction");

        // Check the status before the gift code hits the ledger
        let status = service
            .check_gift_code_status(&gift_code_b58)
            .expect("Could not get gift code status");
        assert_eq!(status, GiftCodeStatus::GiftCodeSubmittedPending);

        // Now add the block with the tx_proposal
        let transaction_log = TransactionLog::log_submitted(
            tx_proposal.clone(),
            14,
            "Gift Code".to_string(),
            Some(&alice_account_id.to_string()),
            &service.wallet_db.get_conn().unwrap(),
        )
        .expect("Could not log submitted");
        add_block_with_tx_proposal(&mut ledger_db, tx_proposal);
        manually_sync_account(
            &ledger_db,
            &service.wallet_db,
            &alice_account_id,
            14,
            &logger,
        );

        // Now the Gift Code should be Available
        let status = service
            .check_gift_code_status(&gift_code_b58)
            .expect("Could not get gift code status");
        assert_eq!(status, GiftCodeStatus::GiftCodeAvailable);

        let transaction_recipient =
            b58_decode(&transaction_log.recipient_public_address_b58).unwrap();

        let decoded = service
            .decode_gift_code(&gift_code_b58)
            .expect("Could not decode gift code");
        let gift_code_account_key = AccountKey::from(&RootIdentity::from(&decoded.root_entropy));
        let gift_code_public_address = gift_code_account_key.default_subaddress();

        assert_eq!(gift_code_public_address, transaction_recipient);

        // Get the tx_out from the ledger and check that it matches expectations
        log::info!(logger, "Retrieving gift code Txo from ledger");
        let tx_out_index = ledger_db
            .get_tx_out_index_by_public_key(&decoded.txo_public_key)
            .unwrap();
        let tx_out = ledger_db.get_tx_out_by_index(tx_out_index).unwrap();
        let shared_secret = get_tx_out_shared_secret(
            gift_code_account_key.view_private_key(),
            &RistrettoPublic::try_from(&tx_out.public_key).unwrap(),
        );
        let (value, _blinding) = tx_out.amount.get_value(&shared_secret).unwrap();
        assert_eq!(value, 2000000000000);

        // Verify balance for Alice = original balance - fee - gift_code_value
        let balance = service
            .get_balance_for_account(&AccountID(alice.account_id_hex.clone()))
            .unwrap();
        assert_eq!(balance.unspent, 97990000000000);

        // Verify that we can get the gift_code
        log::info!(logger, "Getting gift code from database");
        let gotten_gift_code = service.get_gift_code(&gift_code_b58).unwrap();
        assert_eq!(gotten_gift_code.value, value as i64);
        assert_eq!(gotten_gift_code.gift_code_b58, gift_code_b58.to_string());

        // Check that we can list all
        log::info!(logger, "Listing all gift codes");
        let gift_codes = service.list_gift_codes().unwrap();
        assert_eq!(gift_codes.len(), 1);
        assert_eq!(gift_codes[0], gotten_gift_code);

        // Hack to make sure the gift code account has scanned the gift code Txo -
        // otherwise claim_gift_code hangs.
        manually_sync_account(
            &ledger_db,
            &service.wallet_db,
            &AccountID::from(&gift_code_account_key),
            14,
            &logger,
        );

        // Claim the gift code to another account
        log::info!(logger, "Creating new account to receive gift code");
        let bob = service
            .create_account(Some("Bob's Main Account".to_string()), None)
            .unwrap();
        manually_sync_account(
            &ledger_db,
            &service.wallet_db,
            &AccountID(bob.account_id_hex.clone()),
            14,
            &logger,
        );

        log::info!(logger, "Consuming gift code");
        let (consume_response, _gift_code) = service
            .claim_gift_code(&gift_code_b58, &AccountID(bob.account_id_hex.clone()), None)
            .unwrap();

        // Add the consume transaction to the ledger
        log::info!(
            logger,
            "Adding block to ledger with consume gift code transaction"
        );
        {
            let conn = service.wallet_db.get_conn().unwrap();
            let consume_transaction_log =
                TransactionLog::get(&consume_response.transaction_id_hex, &conn).unwrap();
            add_block_from_transaction_log(&mut ledger_db, &conn, &consume_transaction_log);
        };
        manually_sync_account(
            &ledger_db,
            &service.wallet_db,
            &AccountID(bob.account_id_hex.clone()),
            15,
            &logger,
        );

        // Now the Gift Code should be spent
        let status = service
            .check_gift_code_status(&gift_code_b58)
            .expect("Could not get gift code status");
        assert_eq!(status, GiftCodeStatus::GiftCodeClaimed);

        // Bob's balance should be = gift code value - fee (10000000000)
        let bob_balance = service
            .get_balance_for_account(&AccountID(bob.account_id_hex))
            .unwrap();
        assert_eq!(bob_balance.unspent, 1990000000000)
    }
}
