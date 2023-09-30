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
        exclusive_transaction,
        gift_code::GiftCodeModel,
        models::{Account, GiftCode},
        WalletDbError,
    },
    error::WalletTransactionBuilderError,
    service::{
        account::AccountServiceError,
        address::{AddressService, AddressServiceError},
        ledger::{LedgerService, LedgerServiceError},
        models::tx_proposal::TxProposal,
        transaction::{TransactionMemo, TransactionService, TransactionServiceError},
        transaction_builder::DEFAULT_NEW_TX_BLOCK_ATTEMPTS,
        WalletService,
    },
    util::b58::{
        b58_decode_public_address, b58_decode_transfer_payload, b58_encode_public_address,
        b58_encode_transfer_payload, B58Error, DecodedTransferPayload,
    },
};

use mc_account_keys::{AccountKey, DEFAULT_SUBADDRESS_INDEX};
use mc_common::{logger::log, HashSet};
use mc_connection::{BlockchainConnection, RetryableUserTxConnection, UserTxConnection};
use mc_core::slip10::Slip10KeyGenerator;
use mc_crypto_keys::RistrettoPublic;
use mc_crypto_ring_signature_signer::NoKeysRingSigner;
use mc_fog_report_validation::FogPubkeyResolver;
use mc_ledger_db::Ledger;
use mc_transaction_builder::{InputCredentials, RTHMemoBuilder, TransactionBuilder};
use mc_transaction_core::{
    constants::RING_SIZE,
    get_tx_out_shared_secret,
    onetime_keys::recover_onetime_private_key,
    ring_signature::KeyImage,
    tokens::Mob,
    tx::{Tx, TxOut},
    Amount, Token,
};
use mc_transaction_extra::SenderMemoCredential;
use mc_util_uri::FogUri;

use bip39::{Language, Mnemonic, MnemonicType};
use displaydoc::Display;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::json;

use std::{
    convert::TryFrom, fmt, iter::empty, ops::DerefMut, str::FromStr, sync::atomic::Ordering,
};

#[derive(Display, Debug)]
#[allow(clippy::large_enum_variant, clippy::result_large_err)]
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
    TxBuilder(mc_transaction_builder::TxBuilderError),

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

    /// Amount Error: {0}
    Amount(mc_transaction_core::AmountError),

    /// Wallet Transaction Builder Error: {0}
    WalletTransactionBuilder(WalletTransactionBuilderError),

    /// Tx Out Conversion Error: {0}
    TxOutConversion(mc_transaction_core::TxOutConversionError),

    /// Ledger service error: {0}
    LedgerService(LedgerServiceError),

    /// Retry Error
    Retry(mc_connection::RetryError<mc_connection::Error>),
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

impl From<mc_transaction_builder::TxBuilderError> for GiftCodeServiceError {
    fn from(src: mc_transaction_builder::TxBuilderError) -> Self {
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

impl From<AddressServiceError> for GiftCodeServiceError {
    fn from(src: AddressServiceError) -> Self {
        Self::AddressService(src)
    }
}

impl From<mc_transaction_core::AmountError> for GiftCodeServiceError {
    fn from(src: mc_transaction_core::AmountError) -> Self {
        Self::Amount(src)
    }
}

impl From<WalletTransactionBuilderError> for GiftCodeServiceError {
    fn from(src: WalletTransactionBuilderError) -> Self {
        Self::WalletTransactionBuilder(src)
    }
}

impl From<mc_transaction_core::TxOutConversionError> for GiftCodeServiceError {
    fn from(src: mc_transaction_core::TxOutConversionError) -> Self {
        Self::TxOutConversion(src)
    }
}

impl From<LedgerServiceError> for GiftCodeServiceError {
    fn from(src: LedgerServiceError) -> Self {
        Self::LedgerService(src)
    }
}

impl From<mc_connection::RetryError<mc_connection::Error>> for GiftCodeServiceError {
    fn from(src: mc_connection::RetryError<mc_connection::Error>) -> Self {
        Self::Retry(src)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct EncodedGiftCode(pub String);

impl fmt::Display for EncodedGiftCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct DecodedGiftCode {
    pub gift_code_b58: String,
    pub root_entropy: Option<Vec<u8>>,
    pub bip39_entropy: Option<Vec<u8>>,
    pub txo_public_key: Vec<u8>,
    pub value: u64,
    pub memo: String,
}

impl TryFrom<GiftCode> for DecodedGiftCode {
    type Error = GiftCodeServiceError;

    fn try_from(src: GiftCode) -> Result<Self, GiftCodeServiceError> {
        let gift_code = EncodedGiftCode(src.gift_code_b58);
        let transfer_payload = decode_transfer_payload(&gift_code)?;

        Ok(DecodedGiftCode {
            gift_code_b58: gift_code.to_string(),
            root_entropy: transfer_payload.root_entropy.map(|e| e.bytes.to_vec()),
            bip39_entropy: transfer_payload.bip39_entropy,
            txo_public_key: mc_util_serial::encode(&transfer_payload.txo_public_key),
            value: src.value as u64,
            memo: transfer_payload.memo,
        })
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
#[rustfmt::skip]
#[allow(clippy::result_large_err)]
#[async_trait]
pub trait GiftCodeService {
    /// Builds a new gift code.
    ///
    /// # Steps:
    ///  1. Create a new account to receive the funds
    ///  2. Send a transaction to the new account
    ///  3. Wait for the transaction to land
    ///  4. Package the required information into a b58-encoded string
    ///
    /// # Returns:
    /// * JsonSubmitResponse from submitting the gift code transaction to the
    ///   network
    /// * Entropy of the gift code account, hex encoded
    ///
    /// # Arguments
    ///
    ///| Name                  | Purpose                                                            | Notes                                        |
    ///|-----------------------|--------------------------------------------------------------------|----------------------------------------------|
    ///| `from_account_id`     | The account on which to perform this action.                       | Account must exist in the wallet.            |
    ///| `value`               | The amount of MOB to send in this transaction.                     |                                              |
    ///| `memo`                | Memo for whoever claims the gift code.                             |                                              |
    ///| `input_txo_ids`       | The specific TXOs to use as inputs to this transaction.            | TXO IDs (obtain from get_txos_for_account)   |
    ///| `fee`                 | The fee amount to submit with this transaction.                    | If not provided, uses MINIMUM_FEE = .01 MOB. |
    ///| `tombstone_block`     | The block after which this transaction expires.                    | If not provided, uses current height + 10.   |
    ///| `max_spendable_value` | The maximum amount for an input TXO selected for this transaction. |                                              |
    ///
    #[allow(clippy::too_many_arguments)]
    async fn build_gift_code(
        &self,
        from_account_id: &AccountID,
        value: u64,
        memo: Option<String>,
        input_txo_ids: Option<&Vec<String>>,
        fee: Option<u64>,
        tombstone_block: Option<u64>,
        max_spendable_value: Option<u64>,
    ) -> Result<(TxProposal, EncodedGiftCode), GiftCodeServiceError>;

    /// Submit a `tx_proposal` to the ledger that adds the gift code to the wallet_db once the `tx_proposal` has been appended to the ledger.
    ///
    /// # Arguments
    ///
    ///| Name              | Purpose                                      | Notes                                  |
    ///|-------------------|----------------------------------------------|----------------------------------------|
    ///| `from_account_id` | The account on which to perform this action. | Account must exist in the wallet.      |
    ///| `gift_code_b58`   | The base58-encoded gift code contents.       | Must be a valid b58-encoded gift code. |
    ///| `tx_proposal`     | Transaction proposal to submit.              | Created with build_gift_code.          |
    ///
    fn submit_gift_code(
        &self,
        from_account_id: &AccountID,
        gift_code_b58: &EncodedGiftCode,
        tx_proposal: &TxProposal,
    ) -> Result<DecodedGiftCode, GiftCodeServiceError>;

    /// Get the details for a specific gift code.
    ///
    /// # Arguments
    ///
    ///| Name            | Purpose                                | Notes                                  |
    ///|-----------------|----------------------------------------|----------------------------------------|
    ///| `gift_code_b58` | The base58-encoded gift code contents. | Must be a valid b58-encoded gift code. |
    ///
    fn get_gift_code(
        &self,
        gift_code_b58: &EncodedGiftCode,
    ) -> Result<DecodedGiftCode, GiftCodeServiceError>;

    /// List all gift codes in the wallet.
    ///
    /// # Arguments
    ///
    ///| Name         | Purpose                                                  | Notes                    |
    ///|--------------|----------------------------------------------------------|--------------------------|
    ///| `offset`     | The pagination offset. Results start at the offset index | Optional, defaults to 0. |
    ///| `limit`      | Limit for the number of results                          | Optional                 |
    ///
    fn list_gift_codes(
        &self,
        offset: Option<u64>,
        limit: Option<u64>,
    ) -> Result<Vec<DecodedGiftCode>, GiftCodeServiceError>;

    /// Check the status of a gift code currently in your wallet. If the gift
    /// code is not yet in the wallet, add it.
    ///
    /// # Arguments
    ///
    ///| Name            | Purpose                                | Notes                                  |
    ///|-----------------|----------------------------------------|----------------------------------------|
    ///| `gift_code_b58` | The base58-encoded gift code contents. | Must be a valid b58-encoded gift code. |
    ///
    fn check_gift_code_status(
        &self,
        gift_code_b58: &EncodedGiftCode,
    ) -> Result<(GiftCodeStatus, Option<i64>, String), GiftCodeServiceError>;

    /// Execute a transaction from the gift code account to drain the account to
    /// the destination specified by the `account_id_hex` and
    /// `public_address_b58`. If no `public_address_b58` is provided,
    /// then a new `AssignedSubaddress` will be created to receive the funds.
    ///
    /// # Arguments
    ///
    ///| Name            | Purpose                                      | Notes                                  |
    ///|-----------------|----------------------------------------------|----------------------------------------|
    ///| `gift_code_b58` | The base58-encoded gift code contents.       | Must be a valid b58-encoded gift code. |
    ///| `account_id`    | The account on which to perform this action. | Account must exist in the wallet.      |
    ///| `address`       | The public address of the account.           |                                        |
    ///
    fn claim_gift_code(
        &self,
        gift_code_b58: &EncodedGiftCode,
        account_id: &AccountID,
        public_address_b58: Option<String>,
    ) -> Result<Tx, GiftCodeServiceError>;

    ///Remove a gift code from the database.
    ///
    /// # Arguments
    ///
    ///| Name            | Purpose                                | Notes                                  |
    ///|-----------------|----------------------------------------|----------------------------------------|
    ///| `gift_code_b58` | The base58-encoded gift code contents. | Must be a valid b58-encoded gift code. |
    ///
    fn remove_gift_code(
        &self,
        gift_code_b58: &EncodedGiftCode,
    ) -> Result<bool, GiftCodeServiceError>;
}

#[async_trait]
impl<T, FPR> GiftCodeService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    async fn build_gift_code(
        &self,
        from_account_id: &AccountID,
        value: u64,
        memo: Option<String>,
        input_txo_ids: Option<&Vec<String>>,
        fee: Option<u64>,
        tombstone_block: Option<u64>,
        max_spendable_value: Option<u64>,
    ) -> Result<(TxProposal, EncodedGiftCode), GiftCodeServiceError> {
        // First we need to generate a new random bip39 entropy. The way that
        // gift codes work currently is that the sender creates a
        // middleman account and sends that account the amount of MOB
        // desired, plus extra to cover the receivers fee. Then, that
        // account and all of its secrets get encoded into a b58
        // string, and when the receiver gets that they can decode it,
        // and create a new transaction liquidating the gift account of all
        // of the MOB.
        // There should never be a reason to check any other sub_address
        // besides the main one. If there ever is any on a different
        // subaddress, either something went terribly wrong and we
        // messed up, or someone is being very dumb and using a gift
        // account as a place to store their personal MOB.
        let mnemonic = Mnemonic::new(MnemonicType::Words24, Language::English);
        let gift_code_bip39_entropy_bytes = mnemonic.entropy().to_vec();

        let key = mnemonic.derive_slip10_key(0);
        let gift_code_account_key = AccountKey::from(key);

        // We should never actually need this account to exist in the
        // wallet_db, as we will only ever be using it a single time
        // at this instant with a single unspent txo in its main
        // subaddress and the b58 encoded gc will contain all
        // necessary info to generate a tx_proposal for it
        let gift_code_account_main_subaddress_b58 =
            b58_encode_public_address(&gift_code_account_key.default_subaddress())?;

        let mut pooled_conn = self.get_pooled_conn()?;
        let conn = pooled_conn.deref_mut();
        let from_account = Account::get(from_account_id, conn)?;

        let fee_value = fee.map(|f| f.to_string());

        let unsigned_tx_proposal = self.build_transaction(
            &from_account.id,
            &[(
                gift_code_account_main_subaddress_b58,
                crate::json_rpc::v2::models::amount::Amount {
                    value: value.to_string(),
                    token_id: Mob::ID.to_string(),
                },
            )],
            input_txo_ids,
            fee_value,
            None,
            tombstone_block.map(|t| t.to_string()),
            max_spendable_value.map(|f| f.to_string()),
            TransactionMemo::RTH(None, None),
            None,
        )?;

        let tx_proposal = unsigned_tx_proposal.sign(&from_account).await?;

        if tx_proposal.payload_txos.len() != 1 {
            return Err(GiftCodeServiceError::UnexpectedTxProposalFormat);
        }

        let tx_out = &tx_proposal.payload_txos[0].tx_out;

        let proto_tx_pubkey: mc_api::external::CompressedRistretto = (&tx_out.public_key).into();

        let gift_code_b58 = b58_encode_transfer_payload(
            gift_code_bip39_entropy_bytes.to_vec(),
            proto_tx_pubkey,
            memo.unwrap_or_default(),
        )?;

        Ok((tx_proposal, EncodedGiftCode(gift_code_b58)))
    }

    fn submit_gift_code(
        &self,
        from_account_id: &AccountID,
        gift_code_b58: &EncodedGiftCode,
        tx_proposal: &TxProposal,
    ) -> Result<DecodedGiftCode, GiftCodeServiceError> {
        let transfer_payload = decode_transfer_payload(gift_code_b58)?;
        let value = tx_proposal.payload_txos[0].amount.value as i64;

        log::info!(
            self.logger,
            "submitting transaction for gift code... {:?}",
            value
        );

        // Save the gift code to the database before attempting to send it out.
        let mut pooled_conn = self.get_pooled_conn()?;
        let conn = pooled_conn.deref_mut();
        let gift_code =
            exclusive_transaction(conn, |conn| GiftCode::create(gift_code_b58, value, conn))?;

        self.submit_transaction(
            tx_proposal,
            Some(json!({"gift_code_memo": transfer_payload.memo}).to_string()),
            Some(from_account_id.clone().0),
        )?;

        Ok(DecodedGiftCode {
            gift_code_b58: gift_code.gift_code_b58,
            root_entropy: transfer_payload.root_entropy.map(|e| e.bytes.to_vec()),
            bip39_entropy: transfer_payload.bip39_entropy,
            txo_public_key: mc_util_serial::encode(&transfer_payload.txo_public_key),
            value: tx_proposal.payload_txos[0].amount.value,
            memo: transfer_payload.memo,
        })
    }

    fn get_gift_code(
        &self,
        gift_code_b58: &EncodedGiftCode,
    ) -> Result<DecodedGiftCode, GiftCodeServiceError> {
        let mut pooled_conn = self.get_pooled_conn()?;
        let conn = pooled_conn.deref_mut();
        let gift_code = GiftCode::get(gift_code_b58, conn)?;
        DecodedGiftCode::try_from(gift_code)
    }

    fn list_gift_codes(
        &self,
        offset: Option<u64>,
        limit: Option<u64>,
    ) -> Result<Vec<DecodedGiftCode>, GiftCodeServiceError> {
        let mut pooled_conn = self.get_pooled_conn()?;
        let conn = pooled_conn.deref_mut();
        GiftCode::list_all(conn, offset, limit)?
            .into_iter()
            .map(DecodedGiftCode::try_from)
            .collect()
    }

    fn check_gift_code_status(
        &self,
        gift_code_b58: &EncodedGiftCode,
    ) -> Result<(GiftCodeStatus, Option<i64>, String), GiftCodeServiceError> {
        log::info!(self.logger, "encoded_gift_code: {:?}", gift_code_b58);

        let transfer_payload = decode_transfer_payload(gift_code_b58)?;
        let gift_account_key = transfer_payload.account_key;

        log::info!(
            self.logger,
            "transfer_payload.pubKey: {:?}, account_key: {:?}",
            transfer_payload.txo_public_key,
            gift_account_key
        );

        // Check if the GiftCode is in the local ledger.
        let gift_txo = match self
            .ledger_db
            .get_tx_out_index_by_public_key(&transfer_payload.txo_public_key)
        {
            Ok(tx_out_index) => self.ledger_db.get_tx_out_by_index(tx_out_index)?,
            Err(mc_ledger_db::Error::NotFound) => {
                return Ok((
                    GiftCodeStatus::GiftCodeSubmittedPending,
                    None,
                    transfer_payload.memo,
                ))
            }
            Err(e) => return Err(e.into()),
        };

        let shared_secret = get_tx_out_shared_secret(
            gift_account_key.view_private_key(),
            &RistrettoPublic::try_from(&gift_txo.public_key)?,
        );

        let (value, _blinding) = gift_txo.get_masked_amount()?.get_value(&shared_secret)?;

        // Check if the Gift Code has been spent - by convention gift codes are always
        // to the main subaddress index and gift accounts should NEVER have MOB stored
        // anywhere else. If they do, that's not good :,)
        let gift_code_key_image = {
            let onetime_private_key = recover_onetime_private_key(
                &RistrettoPublic::try_from(&transfer_payload.txo_public_key)?,
                gift_account_key.view_private_key(),
                &gift_account_key.subaddress_spend_private(DEFAULT_SUBADDRESS_INDEX),
            );
            KeyImage::from(&onetime_private_key)
        };

        if self.ledger_db.contains_key_image(&gift_code_key_image)? {
            return Ok((
                GiftCodeStatus::GiftCodeClaimed,
                Some(value.value as i64),
                transfer_payload.memo,
            ));
        }

        Ok((
            GiftCodeStatus::GiftCodeAvailable,
            Some(value.value as i64),
            transfer_payload.memo,
        ))
    }

    fn claim_gift_code(
        &self,
        gift_code_b58: &EncodedGiftCode,
        account_id: &AccountID,
        public_address_b58: Option<String>,
    ) -> Result<Tx, GiftCodeServiceError> {
        let (status, gift_value, _memo) = self.check_gift_code_status(gift_code_b58)?;

        match status {
            GiftCodeStatus::GiftCodeClaimed => return Err(GiftCodeServiceError::GiftCodeClaimed),
            GiftCodeStatus::GiftCodeSubmittedPending => {
                return Err(GiftCodeServiceError::GiftCodeNotYetAvailable)
            }
            GiftCodeStatus::GiftCodeAvailable => {}
        }

        let gift_value = gift_value.ok_or(GiftCodeServiceError::GiftCodeNotYetAvailable)?;

        let transfer_payload = decode_transfer_payload(gift_code_b58)?;
        let gift_account_key = transfer_payload.account_key;

        let default_subaddress = if public_address_b58.is_some() {
            public_address_b58.ok_or(GiftCodeServiceError::AccountNotFound)
        } else {
            let address = self.assign_address_for_account(
                account_id,
                Some(&json!({"gift_code_memo": transfer_payload.memo}).to_string()),
            )?;
            Ok(address.public_address_b58)
        }?;

        let recipient_public_address = b58_decode_public_address(&default_subaddress)?;

        // If the gift code value is less than the MINIMUM_FEE, well, then shucks,
        // someone messed up when they were making it. Welcome to the Lost MOB
        // club :)
        if (gift_value as u64) < Mob::MINIMUM_FEE {
            return Err(GiftCodeServiceError::InsufficientValueForFee(
                gift_value as u64,
            ));
        }

        let gift_txo_index = self
            .ledger_db
            .get_tx_out_index_by_public_key(&transfer_payload.txo_public_key)?;

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
        let mut memo_builder = RTHMemoBuilder::default();
        memo_builder.set_sender_credential(SenderMemoCredential::from(&gift_account_key));
        memo_builder.enable_destination_memo();
        let block_version = self.get_network_block_version()?;
        let fee = Amount::new(Mob::MINIMUM_FEE, Mob::ID);
        let mut transaction_builder =
            TransactionBuilder::new(block_version, fee, fog_resolver, memo_builder)?;
        transaction_builder.add_input(input_credentials);
        transaction_builder.add_output(
            Amount::new(gift_value as u64 - Mob::MINIMUM_FEE, Mob::ID),
            &recipient_public_address,
            &mut rng,
        )?;

        let num_blocks_in_ledger = self.ledger_db.num_blocks()?;
        transaction_builder
            .set_tombstone_block(num_blocks_in_ledger + DEFAULT_NEW_TX_BLOCK_ATTEMPTS);
        let tx = transaction_builder.build(&NoKeysRingSigner {}, &mut rng)?;

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

    fn remove_gift_code(
        &self,
        gift_code_b58: &EncodedGiftCode,
    ) -> Result<bool, GiftCodeServiceError> {
        let mut pooled_conn = self.get_pooled_conn()?;
        let conn = pooled_conn.deref_mut();
        exclusive_transaction(conn, |conn| {
            GiftCode::get(gift_code_b58, conn)?.delete(conn)
        })?;
        Ok(true)
    }
}

/// Decode the gift code from b58 to its component parts.
#[allow(clippy::result_large_err)]
pub fn decode_transfer_payload(
    gift_code_b58: &EncodedGiftCode,
) -> Result<DecodedTransferPayload, GiftCodeServiceError> {
    Ok(b58_decode_transfer_payload(gift_code_b58.to_string())?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        service::{account::AccountService, balance::BalanceService},
        test_utils::{
            add_block_to_ledger_db, add_block_with_tx, get_test_ledger, manually_sync_account,
            setup_wallet_service, MOB,
        },
    };
    use mc_account_keys::PublicAddress;
    use mc_common::logger::{async_test_with_logger, Logger};
    use mc_rand::rand_core::RngCore;
    use mc_transaction_core::{ring_signature::KeyImage, tokens::Mob, Token};
    use rand::{rngs::StdRng, SeedableRng};

    #[async_test_with_logger]
    async fn test_gift_code_lifecycle(logger: Logger) {
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
            )
            .unwrap();

        // Add a block with a transaction for Alice
        let alice_account_key: AccountKey = mc_util_serial::decode(&alice.account_key).unwrap();
        let alice_public_address = &alice_account_key.default_subaddress();
        let alice_account_id = AccountID(alice.id.to_string());

        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![alice_public_address.clone()],
            100 * MOB,
            &[KeyImage::from(rng.next_u64())],
            &mut rng,
        );
        manually_sync_account(
            &ledger_db,
            service.wallet_db.as_ref().unwrap(),
            &alice_account_id,
            &logger,
        );

        // Verify balance for Alice
        let balance = service
            .get_balance_for_account(&AccountID(alice.id.clone()))
            .unwrap();
        let balance_pmob = balance.get(&Mob::ID).unwrap();
        assert_eq!(balance_pmob.unspent, 100 * MOB as u128);

        // Create a gift code for Bob
        let (tx_proposal, gift_code_b58) = service
            .build_gift_code(
                &AccountID(alice.id.clone()),
                2 * MOB,
                Some("Gift code for Bob".to_string()),
                None,
                None,
                None,
                None,
            )
            .await
            .unwrap();
        log::info!(logger, "Built gift code transaction");

        let _gift_code = service
            .submit_gift_code(&AccountID(alice.id.clone()), &gift_code_b58, &tx_proposal)
            .unwrap();

        // Check the status before the gift code hits the ledger
        let (status, gift_code_value_opt, _memo) = service
            .check_gift_code_status(&gift_code_b58)
            .expect("Could not get gift code status");
        assert_eq!(status, GiftCodeStatus::GiftCodeSubmittedPending);
        assert!(gift_code_value_opt.is_none());

        add_block_with_tx(&mut ledger_db, tx_proposal.tx, &mut rng);
        manually_sync_account(
            &ledger_db,
            service.wallet_db.as_ref().unwrap(),
            &alice_account_id,
            &logger,
        );

        // Now the Gift Code should be Available
        let (status, gift_code_value_opt, _memo) = service
            .check_gift_code_status(&gift_code_b58)
            .expect("Could not get gift code status");
        assert_eq!(status, GiftCodeStatus::GiftCodeAvailable);
        assert!(gift_code_value_opt.is_some());

        let decoded = decode_transfer_payload(&gift_code_b58).expect("Could not decode gift code");
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
        let (value, _blinding) = tx_out
            .get_masked_amount()
            .unwrap()
            .get_value(&shared_secret)
            .unwrap();
        assert_eq!(value, Amount::new(2 * MOB, Mob::ID));

        // Verify balance for Alice = original balance - fee - gift_code_value
        let balance = service
            .get_balance_for_account(&AccountID(alice.id))
            .unwrap();
        let balance_pmob = balance.get(&Mob::ID).unwrap();
        assert_eq!(balance_pmob.unspent, (98 * MOB - Mob::MINIMUM_FEE) as u128);

        // Verify that we can get the gift_code
        log::info!(logger, "Getting gift code from database");
        let gotten_gift_code = service.get_gift_code(&gift_code_b58).unwrap();
        assert_eq!(gotten_gift_code.value, value.value);
        assert_eq!(gotten_gift_code.gift_code_b58, gift_code_b58.to_string());

        // Check that we can list all
        log::info!(logger, "Listing all gift codes");
        let gift_codes = service.list_gift_codes(None, None).unwrap();
        assert_eq!(gift_codes.len(), 1);
        assert_eq!(gift_codes[0], gotten_gift_code);

        // Claim the gift code to another account
        log::info!(logger, "Creating new account to receive gift code");
        let bob = service
            .create_account(
                Some("Bob's Main Account".to_string()),
                "".to_string(),
                "".to_string(),
            )
            .unwrap();
        manually_sync_account(
            &ledger_db,
            service.wallet_db.as_ref().unwrap(),
            &AccountID(bob.id.clone()),
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
            .claim_gift_code(&gift_code_b58, &AccountID(bob.id.clone()), None)
            .unwrap();

        // Add the consume transaction to the ledger
        log::info!(
            logger,
            "Adding block to ledger with consume gift code transaction"
        );
        add_block_with_tx(&mut ledger_db, tx, &mut rng);
        manually_sync_account(
            &ledger_db,
            service.wallet_db.as_ref().unwrap(),
            &AccountID(bob.id.clone()),
            &logger,
        );

        // Now the Gift Code should be spent
        let (status, gift_code_value_opt, _memo) = service
            .check_gift_code_status(&gift_code_b58)
            .expect("Could not get gift code status");
        assert_eq!(status, GiftCodeStatus::GiftCodeClaimed);
        assert!(gift_code_value_opt.is_some());

        // Bob's balance should be = gift code value - fee (10000000000)
        let bob_balance = service.get_balance_for_account(&AccountID(bob.id)).unwrap();
        let bob_balance_pmob = bob_balance.get(&Mob::ID).unwrap();
        assert_eq!(
            bob_balance_pmob.unspent,
            (2 * MOB - Mob::MINIMUM_FEE) as u128
        )
    }

    #[async_test_with_logger]
    async fn test_remove_gift_code(logger: Logger) {
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
            )
            .unwrap();

        // Add a block with a transaction for Alice
        let alice_account_key: AccountKey = mc_util_serial::decode(&alice.account_key).unwrap();
        let alice_public_address = &alice_account_key.default_subaddress();
        let alice_account_id = AccountID(alice.id.to_string());

        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![alice_public_address.clone()],
            100 * MOB,
            &[KeyImage::from(rng.next_u64())],
            &mut rng,
        );
        manually_sync_account(
            &ledger_db,
            service.wallet_db.as_ref().unwrap(),
            &alice_account_id,
            &logger,
        );

        // Verify balance for Alice
        let balance = service
            .get_balance_for_account(&AccountID(alice.id.clone()))
            .unwrap();
        let balance_pmob = balance.get(&Mob::ID).unwrap();
        assert_eq!(balance_pmob.unspent, 100 * MOB as u128);

        // Create a gift code for Bob
        let (tx_proposal, gift_code_b58) = service
            .build_gift_code(
                &AccountID(alice.id.clone()),
                2 * MOB,
                Some("Gift code for Bob".to_string()),
                None,
                None,
                None,
                None,
            )
            .await
            .unwrap();
        log::info!(logger, "Built gift code transaction");

        let _gift_code = service
            .submit_gift_code(&AccountID(alice.id), &gift_code_b58, &tx_proposal)
            .unwrap();

        // Check the status before the gift code hits the ledger
        let (status, gift_code_value_opt, _memo) = service
            .check_gift_code_status(&gift_code_b58)
            .expect("Could not get gift code status");
        assert_eq!(status, GiftCodeStatus::GiftCodeSubmittedPending);
        assert!(gift_code_value_opt.is_none());

        // Let transaction hit the ledger
        add_block_with_tx(&mut ledger_db, tx_proposal.tx, &mut rng);
        manually_sync_account(
            &ledger_db,
            service.wallet_db.as_ref().unwrap(),
            &alice_account_id,
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
            .list_gift_codes(None, None)
            .expect("Could not list gift codes");
        assert_eq!(gift_codes.len(), 1);

        // remove that gift code
        assert!(service
            .remove_gift_code(&gift_code_b58)
            .expect("Could not remove gift code"));
        let gift_codes = service
            .list_gift_codes(None, None)
            .expect("Could not list gift codes");
        assert_eq!(gift_codes.len(), 0);
    }
}
