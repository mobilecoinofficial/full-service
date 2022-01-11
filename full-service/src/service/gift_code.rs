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
        gift_code::GiftCodeModel,
        models::{Account, GiftCode},
        txo::TxoID,
        WalletDbError,
    },
    service::{
        account::AccountServiceError,
        address::{AddressService, AddressServiceError},
        transaction::{TransactionService, TransactionServiceError},
        WalletService,
    },
    util::b58::{
        b58_decode_public_address, b58_decode_transfer_payload, b58_encode_public_address,
        b58_encode_transfer_payload, B58Error, DecodedTransferPayload,
    },
};
use bip39::{Language, Mnemonic, MnemonicType};
use diesel::Connection;
use displaydoc::Display;
use mc_account_keys::{AccountKey, DEFAULT_SUBADDRESS_INDEX};
use mc_account_keys_slip10::Slip10KeyGenerator;
use mc_common::{logger::log, HashSet};
use mc_connection::{BlockchainConnection, RetryableUserTxConnection, UserTxConnection};
use mc_crypto_keys::RistrettoPublic;
use mc_fog_report_validation::FogPubkeyResolver;
use mc_ledger_db::Ledger;
use mc_mobilecoind::payments::TxProposal;
use mc_transaction_core::{
    constants::{MINIMUM_FEE, RING_SIZE},
    get_tx_out_shared_secret,
    onetime_keys::recover_onetime_private_key,
    ring_signature::KeyImage,
    tx::{Tx, TxOut},
};
use mc_transaction_std::{InputCredentials, NoMemoBuilder, TransactionBuilder};
use mc_util_uri::FogUri;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt, iter::empty, str::FromStr, sync::atomic::Ordering};

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
    ProstDecode(mc_util_serial::DecodeError),

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

    /// The Account is Not Found
    AccountNotFound,

    /** The TxProposal for this GiftCode was constructed in an unexpected
     * manner.
     */
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

    /// Gift Code was removed from the DB prior to claiming
    GiftCodeRemoved,

    /// Node Not Found
    NodeNotFound,

    /// Connection Error
    Connection(retry::Error<mc_connection::Error>),

    /// Error converting to/from API protos: {0}
    ProtoConversion(mc_api::ConversionError),

    /// Error with Transaction Builder
    TxBuilder(mc_transaction_std::TxBuilderError),

    /// Error parsing URI: {0}
    UriParse(mc_util_uri::UriParseError),

    /// Error with Account Service
    AddressService(AddressServiceError),

    /// Error with the B58 Util: {0}
    B58(B58Error),

    /// Error with the FogPubkeyResolver: {0}
    FogPubkeyResolver(String),

    /// Invalid Fog Uri: {0}
    InvalidFogUri(String),
}

impl From<WalletDbError> for GiftCodeServiceError {
    fn from(src: WalletDbError) -> Self {
        Self::Database(src)
    }
}

impl From<B58Error> for GiftCodeServiceError {
    fn from(src: B58Error) -> Self {
        Self::B58(src)
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

impl From<mc_util_serial::DecodeError> for GiftCodeServiceError {
    fn from(src: mc_util_serial::DecodeError) -> Self {
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

impl From<mc_transaction_std::TxBuilderError> for GiftCodeServiceError {
    fn from(src: mc_transaction_std::TxBuilderError) -> Self {
        Self::TxBuilder(src)
    }
}

impl From<mc_api::ConversionError> for GiftCodeServiceError {
    fn from(src: mc_api::ConversionError) -> Self {
        Self::ProtoConversion(src)
    }
}

impl From<mc_util_uri::UriParseError> for GiftCodeServiceError {
    fn from(src: mc_util_uri::UriParseError) -> Self {
        Self::UriParse(src)
    }
}

impl From<retry::Error<mc_connection::Error>> for GiftCodeServiceError {
    fn from(e: retry::Error<mc_connection::Error>) -> Self {
        Self::Connection(e)
    }
}

impl From<AddressServiceError> for GiftCodeServiceError {
    fn from(src: AddressServiceError) -> Self {
        Self::AddressService(src)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct EncodedGiftCode(pub String);

impl fmt::Display for EncodedGiftCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Possible states for a Gift Code in relation to accounts in this wallet.
#[allow(clippy::enum_variant_names)]
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
    ) -> Result<(TxProposal, EncodedGiftCode), GiftCodeServiceError>;

    fn submit_gift_code(
        &self,
        from_account_id: &AccountID,
        gift_code_b58: &EncodedGiftCode,
        tx_proposal: &TxProposal,
    ) -> Result<GiftCode, GiftCodeServiceError>;

    /// Get the details for a specific gift code.
    fn get_gift_code(
        &self,
        gift_code_b58: &EncodedGiftCode,
    ) -> Result<GiftCode, GiftCodeServiceError>;

    /// List all gift codes in the wallet.
    fn list_gift_codes(&self) -> Result<Vec<GiftCode>, GiftCodeServiceError>;

    /// Check the status of a gift code currently in your wallet. If the gift
    /// code is not yet in the wallet, add it.
    fn check_gift_code_status(
        &self,
        gift_code_b58: &EncodedGiftCode,
    ) -> Result<(GiftCodeStatus, Option<i64>, String), GiftCodeServiceError>;

    /// Execute a transaction from the gift code account to drain the account to
    /// the destination specified by the account_id_hex and
    /// assigned_subaddress_b58. If no assigned_subaddress_b58 is provided,
    /// then a new AssignedSubaddress will be created to receive the funds.
    fn claim_gift_code(
        &self,
        gift_code_b58: &EncodedGiftCode,
        account_id: &AccountID,
        assigned_subaddress_b58: Option<String>,
    ) -> Result<Tx, GiftCodeServiceError>;

    /// Decode the gift code from b58 to its component parts.
    fn decode_gift_code(
        &self,
        gift_code_b58: &EncodedGiftCode,
    ) -> Result<DecodedTransferPayload, GiftCodeServiceError>;

    fn remove_gift_code(
        &self,
        gift_code_b58: &EncodedGiftCode,
    ) -> Result<bool, GiftCodeServiceError>;
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
    ) -> Result<(TxProposal, EncodedGiftCode), GiftCodeServiceError> {
        // First we need to generate a new random bip39 entropy. The way that gift codes
        // work currently is that the sender creates a middle_man account and
        // sends that account the amount of MOB desired, plus extra to cover the
        // receivers fee Then, that account and all of its secrets get encoded
        // into a b58 string, and when the receiver gets that they can decode it
        // and create a new transaction liquidating the gift account of all
        // of the MOB on its primary account.
        // There should never be a reason to check any other sub_address besides the
        // main one. If there ever is any on a different subaddress, either
        // something went terribly wrong and we messed up, or someone is being
        // very dumb and using a gift account as a place to store their personal MOB.
        let mnemonic = Mnemonic::new(MnemonicType::Words24, Language::English);
        let gift_code_bip39_entropy_bytes = mnemonic.entropy().to_vec();

        let key = mnemonic.derive_slip10_key(0);
        let gift_code_account_key = AccountKey::from(key);

        // We should never actually need this account to exist in the wallet_db, as we
        // will only ever be using it a single time at this instant with a
        // single unspent txo in its main subaddress and the b58 encoded gc will
        // contain all necessary info to generate a tx_proposal for it
        let gift_code_account_main_subaddress_b58 =
            b58_encode_public_address(&gift_code_account_key.default_subaddress())?;

        let conn = self.wallet_db.get_conn()?;
        let from_account = conn.transaction(|| Account::get(from_account_id, &conn))?;

        let tx_proposal = self.build_transaction(
            &from_account.account_id_hex,
            &[(gift_code_account_main_subaddress_b58, value.to_string())],
            input_txo_ids,
            fee.map(|f| f.to_string()),
            tombstone_block.map(|t| t.to_string()),
            max_spendable_value.map(|f| f.to_string()),
            None,
        )?;

        if tx_proposal.outlay_index_to_tx_out_index.len() != 1 {
            return Err(GiftCodeServiceError::UnexpectedTxProposalFormat);
        }

        let outlay_index = tx_proposal.outlay_index_to_tx_out_index[&0];
        let tx_out = tx_proposal.tx.prefix.outputs[outlay_index].clone();
        let txo_public_key = tx_out.public_key;

        let proto_tx_pubkey: mc_api::external::CompressedRistretto = (&txo_public_key).into();

        let gift_code_b58 = b58_encode_transfer_payload(
            gift_code_bip39_entropy_bytes.to_vec(),
            proto_tx_pubkey,
            memo.unwrap_or_else(|| "".to_string()),
        )?;

        Ok((tx_proposal, EncodedGiftCode(gift_code_b58)))
    }

    fn submit_gift_code(
        &self,
        from_account_id: &AccountID,
        gift_code_b58: &EncodedGiftCode,
        tx_proposal: &TxProposal,
    ) -> Result<GiftCode, GiftCodeServiceError> {
        let decoded_gift_code = self.decode_gift_code(gift_code_b58)?;
        let value = tx_proposal.outlays[0].value as i64;

        log::info!(
            self.logger,
            "submitting transaction for gift code... {:?}",
            value
        );

        // Save the gift code to the database before attempting to send it out.
        let conn = self.wallet_db.get_conn()?;
        let gift_code = conn.transaction(|| {
            GiftCode::create(
                gift_code_b58,
                decoded_gift_code.root_entropy.as_ref(),
                decoded_gift_code.bip39_entropy.as_ref(),
                &decoded_gift_code.txo_public_key,
                value,
                &decoded_gift_code.memo,
                from_account_id,
                &TxoID::from(&tx_proposal.tx.prefix.outputs[0].clone()),
                &conn,
            )
        })?;

        self.submit_transaction(
            tx_proposal.clone(),
            Some(json!({"gift_code_memo": decoded_gift_code.memo}).to_string()),
            Some(from_account_id.clone().0),
        )?;

        Ok(gift_code)
    }

    fn get_gift_code(
        &self,
        gift_code_b58: &EncodedGiftCode,
    ) -> Result<GiftCode, GiftCodeServiceError> {
        let conn = self.wallet_db.get_conn()?;
        conn.transaction(|| Ok(GiftCode::get(gift_code_b58, &conn)?))
    }

    fn list_gift_codes(&self) -> Result<Vec<GiftCode>, GiftCodeServiceError> {
        let conn = self.wallet_db.get_conn()?;
        conn.transaction(|| Ok(GiftCode::list_all(&conn)?))
    }

    fn check_gift_code_status(
        &self,
        gift_code_b58: &EncodedGiftCode,
    ) -> Result<(GiftCodeStatus, Option<i64>, String), GiftCodeServiceError> {
        log::info!(self.logger, "encoded_gift_code: {:?}", gift_code_b58);

        let decoded_gift_code = self.decode_gift_code(gift_code_b58)?;
        let gift_account_key = decoded_gift_code.account_key;

        log::info!(
            self.logger,
            "decoded_gift_code.pubKey: {:?}, account_key: {:?}",
            decoded_gift_code.txo_public_key,
            gift_account_key
        );

        // Check if the GiftCode is in the local ledger.
        let gift_txo = match self
            .ledger_db
            .get_tx_out_index_by_public_key(&decoded_gift_code.txo_public_key)
        {
            Ok(tx_out_index) => self.ledger_db.get_tx_out_by_index(tx_out_index)?,
            Err(mc_ledger_db::Error::NotFound) => {
                return Ok((
                    GiftCodeStatus::GiftCodeSubmittedPending,
                    None,
                    decoded_gift_code.memo,
                ))
            }
            Err(e) => return Err(e.into()),
        };

        let shared_secret = get_tx_out_shared_secret(
            gift_account_key.view_private_key(),
            &RistrettoPublic::try_from(&gift_txo.public_key)?,
        );

        let (value, _blinding) = gift_txo.amount.get_value(&shared_secret).unwrap();

        // Check if the Gift Code has been spent - by convention gift codes are always
        // to the main subaddress index and gift accounts should NEVER have MOB stored
        // anywhere else. If they do, that's not good :,)
        let gift_code_key_image = {
            let onetime_private_key = recover_onetime_private_key(
                &RistrettoPublic::try_from(&decoded_gift_code.txo_public_key)?,
                gift_account_key.view_private_key(),
                &gift_account_key.subaddress_spend_private(DEFAULT_SUBADDRESS_INDEX as u64),
            );
            KeyImage::from(&onetime_private_key)
        };

        if self.ledger_db.contains_key_image(&gift_code_key_image)? {
            return Ok((
                GiftCodeStatus::GiftCodeClaimed,
                Some(value as i64),
                decoded_gift_code.memo,
            ));
        }

        Ok((
            GiftCodeStatus::GiftCodeAvailable,
            Some(value as i64),
            decoded_gift_code.memo,
        ))
    }

    fn claim_gift_code(
        &self,
        gift_code_b58: &EncodedGiftCode,
        account_id: &AccountID,
        assigned_subaddress_b58: Option<String>,
    ) -> Result<Tx, GiftCodeServiceError> {
        let (status, gift_value, _memo) = self.check_gift_code_status(gift_code_b58)?;

        match status {
            GiftCodeStatus::GiftCodeClaimed => return Err(GiftCodeServiceError::GiftCodeClaimed),
            GiftCodeStatus::GiftCodeSubmittedPending => {
                return Err(GiftCodeServiceError::GiftCodeNotYetAvailable)
            }
            GiftCodeStatus::GiftCodeAvailable => {}
        }

        let gift_value = gift_value.unwrap();

        let decoded_gift_code = self.decode_gift_code(gift_code_b58)?;
        let gift_account_key = decoded_gift_code.account_key;

        let default_subaddress = if assigned_subaddress_b58.is_some() {
            assigned_subaddress_b58.ok_or(GiftCodeServiceError::AccountNotFound)
        } else {
            let address = self.assign_address_for_account(
                account_id,
                Some(&json!({"gift_code_memo": decoded_gift_code.memo}).to_string()),
            )?;
            Ok(address.assigned_subaddress_b58)
        }?;

        let recipient_public_address = b58_decode_public_address(&default_subaddress)?;

        // If the gift code value is less than the MINIMUM_FEE, well, then shucks,
        // someone messed up when they were making it. Welcome to the Lost MOB
        // club :)
        if (gift_value as u64) < MINIMUM_FEE {
            return Err(GiftCodeServiceError::InsufficientValueForFee(
                gift_value as u64,
            ));
        }

        let gift_txo_index = self
            .ledger_db
            .get_tx_out_index_by_public_key(&decoded_gift_code.txo_public_key)?;

        let mut ring: Vec<TxOut> = Vec::new();
        let mut rng = rand::thread_rng();

        let fog_resolver = {
            let fog_uri = recipient_public_address
                .fog_report_url()
                .map(FogUri::from_str)
                .transpose()?;
            let mut fog_uris = Vec::new();
            if let Some(uri) = fog_uri {
                fog_uris.push(uri);
            }
            (self.fog_resolver_factory)(fog_uris.as_slice())
                .map_err(GiftCodeServiceError::FogPubkeyResolver)?
        };

        let num_txos = self.ledger_db.num_txos()?;
        let mut sampled_indices: HashSet<u64> = HashSet::default();
        while sampled_indices.len() < RING_SIZE - 1 {
            let index = rng.gen_range(0..num_txos);
            if index == gift_txo_index {
                continue;
            }

            sampled_indices.insert(index);
        }

        let mut sampled_indices_vec: Vec<u64> = sampled_indices.into_iter().collect();
        sampled_indices_vec.insert(0, gift_txo_index);

        let membership_proofs = self
            .ledger_db
            .get_tx_out_proof_of_memberships(&sampled_indices_vec)?;

        for index in sampled_indices_vec.iter() {
            ring.push(self.ledger_db.get_tx_out_by_index(*index)?);
        }

        let real_output = ring[0].clone();

        let onetime_private_key = recover_onetime_private_key(
            &RistrettoPublic::try_from(&real_output.public_key)?,
            gift_account_key.view_private_key(),
            &gift_account_key.subaddress_spend_private(DEFAULT_SUBADDRESS_INDEX),
        );

        let input_credentials = InputCredentials::new(
            ring,
            membership_proofs,
            0,
            onetime_private_key,
            *gift_account_key.view_private_key(),
        )?;

        // Create transaction builder.
        // TODO: After servers that support memos are deployed, use RTHMemoBuilder here
        let memo_builder = NoMemoBuilder::default();
        let mut transaction_builder = TransactionBuilder::new(fog_resolver, memo_builder);
        transaction_builder.add_input(input_credentials);
        let (_tx_out, _confirmation) = transaction_builder.add_output(
            gift_value as u64 - MINIMUM_FEE,
            &recipient_public_address,
            &mut rng,
        )?;

        transaction_builder.set_fee(MINIMUM_FEE)?;

        let num_blocks_in_ledger = self.ledger_db.num_blocks()?;
        transaction_builder.set_tombstone_block(num_blocks_in_ledger + 50);

        let tx = transaction_builder.build(&mut rng)?;

        let responder_ids = self.peer_manager.responder_ids();
        if responder_ids.is_empty() {
            return Err(GiftCodeServiceError::TxoNotConsumable);
        }

        let idx = self.submit_node_offset.fetch_add(1, Ordering::SeqCst);
        let responder_id = &responder_ids[idx % responder_ids.len()];

        let block_index = self
            .peer_manager
            .conn(responder_id)
            .ok_or(GiftCodeServiceError::NodeNotFound)?
            .propose_tx(&tx, empty())?;

        log::info!(
            self.logger,
            "Tx {:?} submitted at block height {}",
            tx,
            block_index
        );

        Ok(tx)
    }

    fn decode_gift_code(
        &self,
        gift_code_b58: &EncodedGiftCode,
    ) -> Result<DecodedTransferPayload, GiftCodeServiceError> {
        Ok(b58_decode_transfer_payload(gift_code_b58.to_string())?)
    }

    fn remove_gift_code(
        &self,
        gift_code_b58: &EncodedGiftCode,
    ) -> Result<bool, GiftCodeServiceError> {
        let conn = self.wallet_db.get_conn()?;
        conn.transaction(|| GiftCode::get(gift_code_b58, &conn)?.delete(&conn))?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        service::{account::AccountService, balance::BalanceService},
        test_utils::{
            add_block_to_ledger_db, add_block_with_tx, add_block_with_tx_proposal, get_test_ledger,
            manually_sync_account, setup_wallet_service, MOB,
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
            .create_account(Some("Alice's Main Account".to_string()))
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
        assert_eq!(balance.unspent, 100 * MOB as u128);

        // Create a gift code for Bob
        let (tx_proposal, gift_code_b58) = service
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
        log::info!(logger, "Built gift code transaction");

        let _gift_code = service
            .submit_gift_code(
                &AccountID(alice.account_id_hex.clone()),
                &gift_code_b58.clone(),
                &tx_proposal.clone(),
            )
            .unwrap();

        // Check the status before the gift code hits the ledger
        let (status, gift_code_value_opt, _memo) = service
            .check_gift_code_status(&gift_code_b58)
            .expect("Could not get gift code status");
        assert_eq!(status, GiftCodeStatus::GiftCodeSubmittedPending);
        assert!(gift_code_value_opt.is_none());

        add_block_with_tx_proposal(&mut ledger_db, tx_proposal);
        manually_sync_account(
            &ledger_db,
            &service.wallet_db,
            &alice_account_id,
            14,
            &logger,
        );

        // Now the Gift Code should be Available
        let (status, gift_code_value_opt, _memo) = service
            .check_gift_code_status(&gift_code_b58)
            .expect("Could not get gift code status");
        assert_eq!(status, GiftCodeStatus::GiftCodeAvailable);
        assert!(gift_code_value_opt.is_some());

        let decoded = service
            .decode_gift_code(&gift_code_b58)
            .expect("Could not decode gift code");
        let gift_code_account_key = decoded.account_key;

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
        assert_eq!(value, 2 * MOB as u64);

        // Verify balance for Alice = original balance - fee - gift_code_value
        let balance = service
            .get_balance_for_account(&AccountID(alice.account_id_hex.clone()))
            .unwrap();
        assert_eq!(balance.unspent, (98 * MOB - MINIMUM_FEE as i64) as u128);

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

        // Claim the gift code to another account
        log::info!(logger, "Creating new account to receive gift code");
        let bob = service
            .create_account(Some("Bob's Main Account".to_string()))
            .unwrap();
        manually_sync_account(
            &ledger_db,
            &service.wallet_db,
            &AccountID(bob.account_id_hex.clone()),
            14,
            &logger,
        );

        // Making sure it doesn't crash when we try to pass in a non-existent account id
        let result = service.claim_gift_code(
            &gift_code_b58,
            &AccountID("nonexistent_account_id".to_string()),
            None,
        );
        assert!(result.is_err());

        let tx = service
            .claim_gift_code(&gift_code_b58, &AccountID(bob.account_id_hex.clone()), None)
            .unwrap();

        // Add the consume transaction to the ledger
        log::info!(
            logger,
            "Adding block to ledger with consume gift code transaction"
        );
        add_block_with_tx(&mut ledger_db, tx);
        manually_sync_account(
            &ledger_db,
            &service.wallet_db,
            &AccountID(bob.account_id_hex.clone()),
            15,
            &logger,
        );

        // Now the Gift Code should be spent
        let (status, gift_code_value_opt, _memo) = service
            .check_gift_code_status(&gift_code_b58)
            .expect("Could not get gift code status");
        assert_eq!(status, GiftCodeStatus::GiftCodeClaimed);
        assert!(gift_code_value_opt.is_some());

        // Bob's balance should be = gift code value - fee (10000000000)
        let bob_balance = service
            .get_balance_for_account(&AccountID(bob.account_id_hex))
            .unwrap();
        assert_eq!(bob_balance.unspent, (2 * MOB - MINIMUM_FEE as i64) as u128)
    }

    #[test_with_logger]
    fn test_remove_gift_code(logger: Logger) {
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
        assert_eq!(balance.unspent, 100 * MOB as u128);

        // Create a gift code for Bob
        let (tx_proposal, gift_code_b58) = service
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
        log::info!(logger, "Built gift code transaction");

        let _gift_code = service
            .submit_gift_code(
                &AccountID(alice.account_id_hex.clone()),
                &gift_code_b58.clone(),
                &tx_proposal.clone(),
            )
            .unwrap();

        // Check the status before the gift code hits the ledger
        let (status, gift_code_value_opt, _memo) = service
            .check_gift_code_status(&gift_code_b58)
            .expect("Could not get gift code status");
        assert_eq!(status, GiftCodeStatus::GiftCodeSubmittedPending);
        assert!(gift_code_value_opt.is_none());

        // Let transaction hit the ledger
        add_block_with_tx_proposal(&mut ledger_db, tx_proposal);
        manually_sync_account(
            &ledger_db,
            &service.wallet_db,
            &alice_account_id,
            14,
            &logger,
        );

        // Check that it landed
        let (status, gift_code_value_opt, _memo) = service
            .check_gift_code_status(&gift_code_b58)
            .expect("Could not get gift code status");
        assert_eq!(status, GiftCodeStatus::GiftCodeAvailable);
        assert!(gift_code_value_opt.is_some());

        // Check that we get all gift codes
        let gift_codes = service
            .list_gift_codes()
            .expect("Could not list gift codes");
        assert_eq!(gift_codes.len(), 1);

        // remove that gift code
        assert!(service
            .remove_gift_code(&gift_code_b58)
            .expect("Could not remove gift code"));
        let gift_codes = service
            .list_gift_codes()
            .expect("Could not list gift codes");
        assert_eq!(gift_codes.len(), 0);
    }
}
