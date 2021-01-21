// Copyright (c) 2020 MobileCoin Inc.

//! Service for managing gift codes.
//!
//! Gift codes are onetime accounts that contain a single TXO. They provide
//! a means to send MOB in a way that can be "claimed," for example, by pasting
//! a QR code for a gift code into a group chat, and the first person to
//! consume the gift code claims the MOB.

use crate::{
    db::{
        account::{AccountID, AccountModel},
        assigned_subaddress::AssignedSubaddressModel,
        b58_encode,
        gift_code::{GiftCodeDbError, GiftCodeModel},
        models::{
            Account, AssignedSubaddress, GiftCode, TransactionLog, Txo, TX_FAILED, TX_PENDING,
            TX_SUCCEEDED,
        },
        transaction_log::TransactionLogModel,
        txo::TxoModel,
    },
    error::{WalletDbError, WalletServiceError},
    service::{
        decorated_types::{JsonGiftCode, JsonSubmitResponse},
        password_manager::PasswordService,
        PasswordServiceError, WalletService,
    },
};
use mc_account_keys::{AccountKey, RootEntropy, RootIdentity};
use mc_common::logger::log;
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_crypto_keys::{CompressedRistrettoPublic, RistrettoPublic};
use mc_fog_report_connection::FogPubkeyResolver;
use mc_ledger_db::Ledger;
use mc_transaction_core::{constants::MINIMUM_FEE, get_tx_out_shared_secret};
use mc_util_from_random::FromRandom;

use diesel::prelude::*;
use displaydoc::Display;
use std::convert::TryFrom;

#[derive(Display, Debug)]
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

    /// Error with the wallet password service: {0}
    PasswordService(PasswordServiceError),

    /// Diesel error: {0}
    Diesel(diesel::result::Error),
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

impl From<PasswordServiceError> for GiftCodeServiceError {
    fn from(src: PasswordServiceError) -> Self {
        Self::PasswordService(src)
    }
}

impl From<diesel::result::Error> for GiftCodeServiceError {
    fn from(src: diesel::result::Error) -> Self {
        Self::Diesel(src)
    }
}

pub struct GiftCodeDetails {
    root_entropy: Vec<u8>,
    txo_public_key: CompressedRistrettoPublic,
    value: u64,
    memo: String,
    account_id: i32,
}

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
    /// * JsonSubmitResponse from submitting the gift code transaction to the network
    /// * Entropy of the gift code account, hex encoded
    #[allow(clippy::too_many_arguments)]
    fn build_and_submit_gift_code(
        &self,
        from_account_id: String,
        value: String,
        name: Option<String>,
        input_txo_ids: Option<&Vec<String>>,
        fee: Option<String>,
        tombstone_block: Option<String>,
        max_spendable_value: Option<String>,
    ) -> Result<(JsonSubmitResponse, String), WalletServiceError>;
    // FIXME: Once we've refactored account to its own service, this can return GiftCodeError

    /// After a gift code has been built and submitted, this method polls for the funded Txo to
    /// hit the ledger, and then constructs the gift code given the entropy, txo_public_key, and
    /// the memo.
    fn register_gift_code(
        &self,
        transaction_log_id: String,
        gift_code_entropy: String,
        poll_interval: Option<u64>,
    ) -> Result<JsonGiftCode, GiftCodeServiceError>;

    /// Get the details for a specific gift code.
    fn get_gift_code(&self, gift_code_b58: String) -> Result<JsonGiftCode, GiftCodeServiceError>;

    /// List all gift codes in the wallet.
    fn list_gift_codes(&self) -> Result<Vec<JsonGiftCode>, GiftCodeServiceError>;

    /// Check the status of a gift code currently in your wallet.
    fn check_gift_code_status(&self) -> Result<(), GiftCodeServiceError>;

    /// Execute a transaction from the gift code account to drain the account to the destination
    /// specified by the account_id_hex and assigned_subaddress_b58. If no assigned_subaddress_b58
    /// is provided, then a new AssignedSubaddress will be created to receive the funds.
    fn consume_gift_code(
        &self,
        gift_code_b58: String,
        account_id_hex: String,
        assigned_subaddress_b58: Option<String>,
    ) -> Result<(JsonSubmitResponse, GiftCodeDetails), WalletServiceError>;
    // FIXME: Once we've refactored transaction building to its own service, this can return GiftCodeError

    ///
    fn register_consumed(
        &self,
        gift_code_b58: String,
        gift_code_details: GiftCodeDetails,
        transaction_log_id: String,
        poll_interval: Option<u64>,
    ) -> Result<JsonGiftCode, GiftCodeServiceError>;
}

impl<T, FPR> GiftCodeService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn build_and_submit_gift_code(
        &self,
        from_account_id: String,
        value: String,
        memo: Option<String>,
        input_txo_ids: Option<&Vec<String>>,
        fee: Option<String>,
        tombstone_block: Option<String>,
        max_spendable_value: Option<String>,
    ) -> Result<(JsonSubmitResponse, String), WalletServiceError> {
        self.verify_unlocked()?;
        // First, create the account which will receive the funds, from a new random entropy
        let mut rng = rand::thread_rng();
        let root_id = RootIdentity::from_random(&mut rng);
        let entropy_str = hex::encode(&root_id.root_entropy);

        // Set first_block to current block height, since we know this account has only existed since now
        let block_height = self.ledger_db.num_blocks()? - 1;
        log::debug!(
            self.logger,
            "Created gift code account. Importing to wallet at block height {:?}.",
            block_height,
        );
        let json_account =
            self.import_account(entropy_str.clone(), memo.clone(), Some(block_height - 1))?;

        // Scope the connection so that we can later poll without keeping this connection open.
        let (gift_code_account, gift_code_account_key, from_account) = {
            let conn_context = self.wallet_db.get_conn_context()?;
            // Send a transaction to the gift_code account
            let (gift_code_account, gift_code_account_key, from_account) =
                conn_context
                    .conn
                    .transaction::<(Account, AccountKey, Account), WalletServiceError, _>(|| {
                        let from_account =
                            Account::get(&AccountID(from_account_id), &conn_context.conn)?;
                        let gift_code_account =
                            Account::get(&AccountID(json_account.account_id), &conn_context.conn)?;

                        let gift_code_account_key =
                            gift_code_account.get_decrypted_account_key(&conn_context)?;
                        log::debug!(
                            self.logger,
                            "Funding gift code account {:?} from account {:?}",
                            gift_code_account.account_id_hex,
                            from_account.account_id_hex,
                        );
                        Ok((gift_code_account, gift_code_account_key, from_account))
                    })?;
            (gift_code_account, gift_code_account_key, from_account)
        };

        println!(
            "\x1b[1;33m Sending to gift code at subaddress {:?}\x1b[0m",
            gift_code_account.main_subaddress_index
        );
        let main_subaddress =
            gift_code_account_key.subaddress(gift_code_account.main_subaddress_index as u64);
        let gift_code_address = b58_encode(&main_subaddress)?;
        println!(
            "\x1b[1;34m Note that gives us spend public key {:?}\x1b[0m",
            main_subaddress.spend_public_key()
        );

        let submit_response = self.send_transaction(
            &from_account.account_id_hex,
            &gift_code_address,
            value,
            input_txo_ids,
            fee,
            tombstone_block,
            max_spendable_value,
            memo,
        )?;

        Ok((submit_response, entropy_str))
    }

    fn register_gift_code(
        &self,
        transaction_id_hex: String,
        gift_code_entropy: String,
        poll_interval: Option<u64>,
    ) -> Result<JsonGiftCode, GiftCodeServiceError> {
        // Poll until we see this transaction land.
        log::debug!(
            self.logger,
            "Now polling for gift code txo to land in ledger"
        );
        let transaction_log = loop {
            let transaction_log =
                { TransactionLog::get(&transaction_id_hex, &self.wallet_db.get_conn()?)? };
            match transaction_log.status.as_str() {
                TX_PENDING => {
                    log::trace!(
                        self.logger,
                        "Gift code txo still pending at block height {:?}. Sleeping.",
                        self.ledger_db.num_blocks()?,
                    );
                    std::thread::sleep(std::time::Duration::from_secs(poll_interval.unwrap_or(5)))
                }
                TX_FAILED => return Err(GiftCodeServiceError::BuildGiftCodeFailed),
                TX_SUCCEEDED => break transaction_log,
                _ => {
                    return Err(GiftCodeServiceError::UnexpectedTxStatus(
                        transaction_log.status,
                    ))
                }
            }
        };

        let conn_context = self.wallet_db.get_conn_context()?;
        let gift_code = conn_context
            .conn
            .transaction::<String, GiftCodeServiceError, _>(|| {
                // Get the Txo Associated with this Transaction
                let txos = transaction_log.get_associated_txos(&conn_context.conn)?;
                if txos.outputs.len() != 1 {
                    return Err(GiftCodeServiceError::UnexpectedNumOutputs(
                        txos.outputs.len(),
                    ));
                }
                let txo = Txo::get(&txos.outputs[0], &conn_context)?;
                let txo_public_key: CompressedRistrettoPublic =
                    mc_util_serial::decode(&txo.txo.public_key)?;

                let mut entropy = [0u8; 32];
                entropy.copy_from_slice(&hex::decode(gift_code_entropy.clone())?);
                let root_id = RootIdentity::from(&entropy);
                let account_key = AccountKey::from(&root_id);
                let gift_code_account =
                    Account::get(&AccountID::from(&account_key), &conn_context.conn)?;

                // Now that the Gift Code is funded, we can add it to our Gift Codes table
                let gift_code = GiftCode::create(
                    &RootEntropy::from(&entropy),
                    &txo_public_key,
                    transaction_log.value,
                    transaction_log.comment.clone(),
                    gift_code_account.id,
                    Some(transaction_log.id),
                    None,
                    &conn_context,
                )?;
                Ok(gift_code)
            })?;

        Ok(JsonGiftCode {
            object: "gift_code".to_string(),
            gift_code,
            entropy: gift_code_entropy,
            value: transaction_log.value.to_string(),
            memo: transaction_log.comment,
        })
    }

    fn get_gift_code(&self, gift_code_b58: String) -> Result<JsonGiftCode, GiftCodeServiceError> {
        self.verify_unlocked()?;

        let conn_context = self.wallet_db.get_conn_context()?;
        let gift_code = GiftCode::get(&gift_code_b58, &conn_context.conn)?;

        Ok(JsonGiftCode {
            object: "gift_code".to_string(),
            gift_code: gift_code_b58,
            entropy: hex::encode(&gift_code.get_decrypted_entropy(&conn_context)?),
            value: gift_code.value.to_string(),
            memo: gift_code.memo,
        })
    }

    fn list_gift_codes(&self) -> Result<Vec<JsonGiftCode>, GiftCodeServiceError> {
        self.verify_unlocked()?;

        let conn_context = self.wallet_db.get_conn_context()?;
        let gift_codes = GiftCode::list_all(&conn_context.conn)?;
        Ok(gift_codes
            .iter()
            .map(|g| JsonGiftCode {
                object: "gift_code".to_string(),
                gift_code: g.gift_code_b58.clone(),
                entropy: hex::encode(
                    &g.get_decrypted_entropy(&conn_context)
                        .expect("Could not decrypt entropy"),
                ),
                value: g.value.to_string(),
                memo: g.memo.clone(),
            })
            .collect())
    }

    fn check_gift_code_status(&self) -> Result<(), GiftCodeServiceError> {
        Ok(())
    }

    fn consume_gift_code(
        &self,
        gift_code_b58: String,
        account_id_hex: String,
        assigned_subaddress_b58: Option<String>,
    ) -> Result<(JsonSubmitResponse, GiftCodeDetails), WalletServiceError> {
        self.verify_unlocked()?;
        log::info!(
            self.logger,
            "Consuming gift code {:?} to account_id {:?} at address {:?}",
            gift_code_b58,
            account_id_hex,
            assigned_subaddress_b58
        );

        // Get the components of the gift code from the printable wrapper
        let wrapper =
            mc_mobilecoind_api::printable::PrintableWrapper::b58_decode(gift_code_b58).unwrap();
        let transfer_payload = wrapper.get_transfer_payload();
        let mut root_entropy = [0u8; 32];
        root_entropy.copy_from_slice(transfer_payload.get_entropy());
        let txo_public_key =
            CompressedRistrettoPublic::try_from(transfer_payload.get_tx_out_public_key()).unwrap();
        let memo = transfer_payload.get_memo();

        // Get the block height to start scanning based on the block index of the tx_out_public_key
        let tx_out_index = self
            .ledger_db
            .get_tx_out_index_by_public_key(&txo_public_key)?;
        let scan_block = self
            .ledger_db
            .get_block_index_by_tx_out_index(tx_out_index)?;

        // Get the value of the txo in the gift
        let gift_txo = self.ledger_db.get_tx_out_by_index(tx_out_index)?;
        let root_id = RootIdentity::from(&root_entropy);
        let gift_code_account_key = AccountKey::from(&root_id);
        let shared_secret = get_tx_out_shared_secret(
            gift_code_account_key.view_private_key(),
            &RistrettoPublic::try_from(&gift_txo.public_key).unwrap(),
        );
        let (value, _blinding) = gift_txo.amount.get_value(&shared_secret).unwrap();

        // Add this account to our DB. It will be drained immediately.
        let (gift_code_account_id_hex, gift_code_account_id) = match Account::get(
            &AccountID::from(&gift_code_account_key),
            &self.wallet_db.get_conn()?,
        ) {
            // The account may already be in the wallet if we constructed this gift code in this wallet.
            Ok(account) => (account.account_id_hex, account.id),
            Err(WalletDbError::AccountNotFound(_)) => {
                let json_gift_code_account = self.import_account(
                    hex::encode(root_entropy),
                    Some(format!("Gift Code: {}", memo)),
                    Some(scan_block),
                )?;
                log::info!(
                    self.logger,
                    "Imported gift code account {:?}.",
                    json_gift_code_account.account_id
                );
                (
                    json_gift_code_account.account_id,
                    json_gift_code_account.offset_count,
                )
            }
            Err(e) => return Err(e.into()),
        };

        // Construct a transaction from the gift code account to our desired recipient account.
        let destination_address = assigned_subaddress_b58.unwrap_or_else(|| {
            let json_address = self
                .create_assigned_subaddress(&account_id_hex, Some(&format!("Gift Code: {}", memo)))
                .unwrap();
            json_address.public_address
        });
        log::info!(
            self.logger,
            "Consuming gift code to destination address {:?}",
            destination_address
        );

        if value < MINIMUM_FEE {
            return Err(GiftCodeServiceError::InsufficientValueForFee(value).into());
        }

        // Sanity check that we have assigned subaddresses for the gift code account
        let addresses =
            AssignedSubaddress::list_all(&gift_code_account_id_hex, &self.wallet_db.get_conn()?)?;
        println!("\x1b[1;33m GOT ADDRESSES: {:?}\x1b[0m", addresses);
        assert_eq!(addresses.len(), 2);

        // Sanity check that our txo is available and spendable from the gift code account
        let txos = Txo::list_for_account(
            &gift_code_account_id_hex,
            &self.wallet_db.get_conn_context()?,
        )?;
        if txos.len() != 1 {
            return Err(
                GiftCodeServiceError::UnexpectedNumTxosInGiftCodeAccount(txos.len()).into(),
            );
        }
        if txos[0].txo.value as u64 != value {
            return Err(GiftCodeServiceError::UnexpectedValueInGiftCodeTxo(
                txos[0].txo.value as u64,
            )
            .into());
        }
        let mut txo = txos[0].clone();
        let max_polling = 3;
        let mut count = 0;
        while txo.txo.subaddress_index.is_none() && count <= max_polling {
            if count == max_polling {
                return Err(GiftCodeServiceError::TxoNotConsumable.into());
            }
            // Note that we now need to allow the sync thread to catch up for this TXO so that we can
            // make sure the subaddress is assigned, rendering the Txo spendable.
            std::thread::sleep(std::time::Duration::from_secs(3));
            log::info!(
                self.logger,
                "\x1b[1;36m Not yet spendable for account {:?}. Txo = {:?}\x1b[0m",
                gift_code_account_id_hex,
                txo
            );
            let txos = Txo::list_for_account(
                &gift_code_account_id_hex,
                &self.wallet_db.get_conn_context()?,
            )?;
            txo = txos[0].clone();
            count += 1;
        }

        log::info!(self.logger, "\x1b[1;33m GOT TXOS = {:?}\x1b[0m", txos);

        // We go with all the defaults because there is only one TXO in this account to spend.
        let submit_response = self.send_transaction(
            &gift_code_account_id_hex,
            &destination_address,
            (value - MINIMUM_FEE).to_string(),
            None,
            Some(MINIMUM_FEE.to_string()),
            None,
            None,
            Some(format!("Consume Gift Code: {}", memo)),
        )?;
        log::info!(
            self.logger,
            "Submitted transaction to consume gift code with id {:?}",
            submit_response.transaction_id
        );
        let details = GiftCodeDetails {
            root_entropy: root_entropy.to_vec(),
            txo_public_key,
            value,
            memo: memo.to_string(),
            account_id: gift_code_account_id,
        };
        Ok((submit_response, details))
    }

    fn register_consumed(
        &self,
        gift_code_b58: String,
        gift_code_details: GiftCodeDetails,
        transaction_log_id: String,
        poll_interval: Option<u64>,
    ) -> Result<JsonGiftCode, GiftCodeServiceError> {
        // FIXME: duplicated fragment
        let transaction_log = loop {
            let transaction_log = {
                let conn = self.wallet_db.get_conn()?;
                TransactionLog::get(&transaction_log_id, &conn)?
            };
            match transaction_log.status.as_str() {
                TX_PENDING => {
                    log::trace!(
                        self.logger,
                        "Gift code txo still pending at block height {:?}. Sleeping.",
                        self.ledger_db.num_blocks()?,
                    );
                    std::thread::sleep(std::time::Duration::from_secs(poll_interval.unwrap_or(5)))
                }
                TX_FAILED => return Err(GiftCodeServiceError::BuildGiftCodeFailed),
                TX_SUCCEEDED => break transaction_log,
                _ => {
                    return Err(GiftCodeServiceError::UnexpectedTxStatus(
                        transaction_log.status,
                    ))
                }
            }
        };
        log::info!(self.logger, "Got transaction log {:?}", transaction_log);

        // Add the consumed gift code to our gift code store. If we also own this gift code, then update.
        let conn = self.wallet_db.get_conn()?;
        match GiftCode::get(&gift_code_b58, &conn) {
            Ok(gc) => {
                log::info!(self.logger, "Updating existing gift code consume_log_id");
                gc.update_consume_log_id(transaction_log.id, &conn)?
            }
            Err(WalletDbError::GiftCode(GiftCodeDbError::GiftCodeNotFound(_))) => {
                log::info!(self.logger, "Registering gift code");
                let entropy: RootEntropy = mc_util_serial::decode(&gift_code_details.root_entropy)?;
                let _gift_code_b58 = GiftCode::create(
                    &entropy,
                    &gift_code_details.txo_public_key,
                    gift_code_details.value as i64,
                    gift_code_details.memo.to_string(),
                    gift_code_details.account_id,
                    None,
                    Some(transaction_log.id),
                    &self.wallet_db.get_conn_context()?,
                )?;
            }
            Err(e) => return Err(e.into()),
        }

        log::info!(self.logger, "Updated gift code");
        Ok(JsonGiftCode {
            object: "gift_code".to_string(),
            gift_code: gift_code_b58,
            entropy: hex::encode(&gift_code_details.root_entropy),
            value: gift_code_details.value.to_string(),
            memo: gift_code_details.memo,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db::b58_decode,
        test_utils::{
            add_block_from_transaction_log, add_block_to_ledger_db, get_test_ledger, setup_service,
            MOB,
        },
    };
    use mc_account_keys::PublicAddress;
    use mc_common::logger::{test_with_logger, Logger};
    use mc_crypto_rand::rand_core::RngCore;
    use mc_transaction_core::ring_signature::KeyImage;
    use rand::{rngs::StdRng, SeedableRng};
    use std::time::Duration;

    #[test_with_logger]
    fn test_gift_code_lifecycle(logger: Logger) {
        let mut rng: StdRng = SeedableRng::from_seed([20u8; 32]);

        let known_recipients: Vec<PublicAddress> = Vec::new();
        let mut ledger_db = get_test_ledger(5, &known_recipients, 12, &mut rng);

        let service = setup_service(ledger_db.clone(), logger.clone());

        let mut password_hash = [0u8; 32];
        rng.fill_bytes(&mut password_hash);
        let res = service.set_password(hex::encode(&password_hash)).unwrap();
        assert!(res);

        // Create our main account for the wallet
        let alice = service
            .create_account(Some("Alice's Main Account".to_string()), None)
            .unwrap();

        // Add a block with a transaction for Alice
        let alice_public_address = b58_decode(&alice.account.main_address).unwrap();
        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![alice_public_address.clone()],
            100 * MOB as u64,
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        // Sleep to let the sync thread process the txo - FIXME poll instead of sleep
        std::thread::sleep(Duration::from_secs(8));

        // Verify balance for Alice
        let balance = service.get_balance(&alice.account.account_id).unwrap();
        assert_eq!(balance.unspent, "100000000000000"); // 100 MOB

        // Create a gift code for Bob
        let (submit_response, gift_code_entropy) = service
            .build_and_submit_gift_code(
                alice.account.account_id.clone(),
                "2000000000000".to_string(), // 2 MOB
                Some("Gift code for Bob".to_string()),
                None,
                None,
                None,
                None,
            )
            .unwrap();
        log::info!(logger, "Built and submitted gift code transaction");

        let json_transaction_log = service
            .get_transaction(&submit_response.transaction_id)
            .unwrap();
        let gift_code_public_address =
            b58_decode(&json_transaction_log.recipient_address_id.unwrap()).unwrap();

        // NOTE: Submitting to the test ledger via propose_tx doesn't actually add the block to the
        //        ledger, because no consensus is occurring, so this is the workaround.
        let transaction_log = {
            let conn = service.wallet_db.get_conn().unwrap();

            TransactionLog::get(&json_transaction_log.transaction_log_id, &conn).unwrap()
        };

        {
            log::info!(logger, "Adding block from transaction log");
            let conn_context = service.wallet_db.get_conn_context().unwrap();
            add_block_from_transaction_log(&mut ledger_db, &conn_context, &transaction_log);
        }

        log::info!(logger, "Registering gift code");
        let gift_code = service
            .register_gift_code(
                submit_response.transaction_id,
                gift_code_entropy.clone(),
                None,
            )
            .unwrap();
        assert_eq!(gift_code_entropy, gift_code.entropy);

        // Get the components of the gift code from the printable wrapper
        log::info!(logger, "Reading gift code b58");
        let wrapper = mc_mobilecoind_api::printable::PrintableWrapper::b58_decode(
            gift_code.gift_code.clone(),
        )
        .unwrap();
        assert!(wrapper.has_transfer_payload());
        let transfer_payload = wrapper.get_transfer_payload();
        let mut root_entropy = [0u8; 32];
        root_entropy.copy_from_slice(transfer_payload.get_entropy());
        let root_id = RootIdentity::from(&root_entropy);
        let gift_code_account_key = AccountKey::from(&root_id);
        let tx_out_public_key =
            CompressedRistrettoPublic::try_from(transfer_payload.get_tx_out_public_key()).unwrap();

        assert_eq!(
            gift_code_account_key.subaddress(0),
            gift_code_public_address
        );

        // Get the tx_out from the ledger and check that it matches expectations
        log::info!(logger, "Retrieving gift code Txo from ledger");
        let tx_out_index = ledger_db
            .get_tx_out_index_by_public_key(&tx_out_public_key)
            .unwrap();
        let tx_out = ledger_db.get_tx_out_by_index(tx_out_index).unwrap();
        let shared_secret = get_tx_out_shared_secret(
            gift_code_account_key.view_private_key(),
            &RistrettoPublic::try_from(&tx_out.public_key).unwrap(),
        );
        let (value, _blinding) = tx_out.amount.get_value(&shared_secret).unwrap();
        assert_eq!(value, 2000000000000);

        // Verify balance for gift code
        log::info!(logger, "Checking gift code balance");
        let gift_code_balance = service
            .get_balance(&AccountID::from(&gift_code_account_key).to_string())
            .unwrap();
        assert_eq!(gift_code_balance.unspent, "2000000000000");

        // Verify balance for Alice = original balance - fee - gift_code_value
        let balance = service.get_balance(&alice.account.account_id).unwrap();
        assert_eq!(balance.unspent, "97990000000000");

        // Verify that we can get the gift_code
        log::info!(logger, "Getting gift code from database");
        let gotten_gift_code = service.get_gift_code(gift_code.gift_code.clone()).unwrap();
        assert_eq!(gotten_gift_code, gift_code);

        // Check that we can list all
        log::info!(logger, "Listing all gift codes");
        let gift_codes = service.list_gift_codes().unwrap();
        assert_eq!(gift_codes.len(), 1);
        assert_eq!(gift_codes[0], gift_code);

        // FIXME: check status for new gift code

        // Consume the gift code to another account
        log::info!(logger, "Creating new account to receive gift code");
        let bob = service
            .create_account(Some("Bob's Main Account".to_string()), None)
            .unwrap();

        log::info!(logger, "Consuming gift code");
        let (consume_response, gift_code_details) = service
            .consume_gift_code(
                gift_code.gift_code.clone(),
                bob.account.account_id.clone(),
                None,
            )
            .unwrap();

        // Add the consume transaction to the ledger
        log::info!(
            logger,
            "Adding block to ledger with consume gift code transaction"
        );
        let consume_transaction_log = {
            let conn_context = service.wallet_db.get_conn_context().unwrap();
            let consume_transaction_log =
                TransactionLog::get(&consume_response.transaction_id, &conn_context.conn).unwrap();
            add_block_from_transaction_log(&mut ledger_db, &conn_context, &consume_transaction_log);
            consume_transaction_log
        };

        log::info!(logger, "Registering consumed");
        service
            .register_consumed(
                gift_code.gift_code,
                gift_code_details,
                consume_transaction_log.transaction_id_hex,
                None,
            )
            .unwrap();

        // Gift code balance should be 0
        let gift_balance = service
            .get_balance(&AccountID::from(&gift_code_account_key).to_string())
            .unwrap();
        assert_eq!(gift_balance.unspent, "0");

        // Bob's balance should be = gift code value - fee (10000000000)
        let bob_balance = service.get_balance(&bob.account.account_id).unwrap();
        assert_eq!(bob_balance.unspent, "1990000000000")

        // FIXME: Test with explicit recipient address
    }
}
