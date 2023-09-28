// Copyright (c) 2020-2021 MobileCoin Inc.

use crate::{db::gift_code::GiftCodeDbError, util::b58::B58Error};
use base64::DecodeSliceError;
use reqwest;

use displaydoc::Display;

#[derive(Display, Debug)]
pub enum WalletDbError {
    /// Wallet functions are currently disabled
    WalletFunctionsDisabled,

    /// View Only Account already exists: {0}
    ViewOnlyAccountAlreadyExists(String),

    /// Account already exists: {0}
    AccountAlreadyExists(String),

    /// Diesel Error: {0}
    Diesel(diesel::result::Error),

    /// Error with rocket databases: {0}
    RocketDB(rocket_sync_db_pools::r2d2::Error),

    /// Duplicate entries with the same ID: {0}
    DuplicateEntries(String),

    /// Error encoding b58: {0}
    B58Encode(mc_api::display::Error),

    /// Error decoding b58: No public address in wrapper.
    B58Decode,

    /// Constructed a malformed transaction with multiple account IDs
    MultipleAccountIDsInTransaction,

    /// Constructed a transaction with no recipient
    TransactionLacksRecipient,

    /** Constructed a transaction that is not linked to any account in the
     * wallet
     */
    TransactionLacksAccount,

    /// Error decoding prost: {0}
    ProstDecode(mc_util_serial::DecodeError),

    /// We expect one change output per TxProposal
    UnexpectedNumberOfChangeOutputs,

    /// Key Image missing when recovering orphaned Txo
    MissingKeyImage,

    /// Subaddress on received transaction is null
    NullSubaddressOnReceived,

    /// No unspent Txos of token id {0} in wallet
    NoSpendableTxos(String),

    /** Txos are too fragmented to construct a transaction with MAX_INPUTS.
     * Please combine txos.
     */
    InsufficientFundsFragmentedTxos,

    /// Insufficient Funds: {0}
    InsufficientFunds(String),

    /// Insufficient funds from Txos under max_spendable_value: {0}
    InsufficientFundsUnderMaxSpendable(String),

    /// Invalid argument for query
    InvalidArgument(String),

    /// Unexpected TXO Type: {0}
    UnexpectedTransactionTxoType(String),

    /// Unexpected AccountTxoStatus: {0}
    UnexpectedAccountTxoStatus(String),

    /// Unexpected number of accounts associated with Txo: {0}
    UnexpectedNumberOfAccountsAssociatedWithTxo(String),

    /// Transaction mismatch when retrieving associated Txos
    TransactionMismatch,

    /// Account Not Found: {0}
    AccountNotFound(String),

    /// AssignedSubaddress Not Found: {0}
    AssignedSubaddressNotFound(String),

    /// Txo Not Found: {0}
    TxoNotFound(String),

    /// TransactionLog Not Found: {0}
    TransactionLogNotFound(String),

    /// AccountTxoStatus not found: {0}
    AccountTxoStatusNotFound(String),

    /// Cannot log a transaction with a value > i64::MAX
    TransactionValueExceedsMax,

    /// The Txo Exists, but for another account: {0}
    TxoExistsForAnotherAccount(String),

    /// The Txo is associated with too many Accounts: {0}
    TxoAssociatedWithTooManyAccounts(String),

    /// The Txo has neither received_to nor spent_from specified.
    MalformedTxoDatabaseEntry,

    /// The account key and the entropy provided to create account do not match.
    AccountSecretsDoNotMatch,

    /** The account cannot be created without either an entropy or an account
     * key.
     */
    InsufficientSecretsToCreateAccount,

    /// Error with the GiftCode service: {0}
    GiftCode(GiftCodeDbError),

    /// Error with the B58 Util: {0}
    B58(B58Error),

    /// Error with the LedgerDB
    LedgerDB(mc_ledger_db::Error),

    /// Error converting to/from API protos: {0}
    ProtoConversion(mc_api::ConversionError),

    /// Error while generating a Slip10Key: {0}
    Slip10Key(mc_account_keys::Error),

    /// Decode from Base64 error: {0}
    Base64Decode(base64::DecodeError),

    /// Subaddresses are not supported for FOG enabled accounts
    SubaddressesNotSupportedForFOGEnabledAccounts,

    /// error converting keys
    KeyError(mc_crypto_keys::KeyError),

    /// invalid txo status
    InvalidTxoStatus(String),

    /// Expected to find TxOut as an outlay
    ExpectedTxOutAsOutlay,

    /// Expected to find a membership proof for txo with id: {0}
    MissingTxoMembershipProof(String),

    /// Expected to find a key image for a txo with id: {0}
    MissingKeyImageForInputTxo(String),

    /// Reqwest library errors
    ReqwestError(reqwest::Error),

    /// ed25519-dalek error
    Dalek(ed25519_dalek::ed25519::Error),

    /// Account key is not available for a view only account
    AccountKeyNotAvailableForViewOnlyAccount,

    /// Decode Slice Error
    DecodeSlice(DecodeSliceError),
}

impl From<diesel::result::Error> for WalletDbError {
    fn from(src: diesel::result::Error) -> Self {
        Self::Diesel(src)
    }
}

impl From<rocket_sync_db_pools::r2d2::Error> for WalletDbError {
    fn from(src: rocket_sync_db_pools::r2d2::Error) -> Self {
        Self::RocketDB(src)
    }
}

impl From<mc_api::ConversionError> for WalletDbError {
    fn from(src: mc_api::ConversionError) -> Self {
        Self::ProtoConversion(src)
    }
}

impl From<mc_api::display::Error> for WalletDbError {
    fn from(src: mc_api::display::Error) -> Self {
        Self::B58Encode(src)
    }
}

impl From<mc_util_serial::DecodeError> for WalletDbError {
    fn from(src: mc_util_serial::DecodeError) -> Self {
        Self::ProstDecode(src)
    }
}

impl From<GiftCodeDbError> for WalletDbError {
    fn from(src: GiftCodeDbError) -> Self {
        Self::GiftCode(src)
    }
}

impl From<B58Error> for WalletDbError {
    fn from(src: B58Error) -> Self {
        Self::B58(src)
    }
}

impl From<mc_ledger_db::Error> for WalletDbError {
    fn from(src: mc_ledger_db::Error) -> Self {
        Self::LedgerDB(src)
    }
}

impl From<mc_account_keys::Error> for WalletDbError {
    fn from(src: mc_account_keys::Error) -> Self {
        Self::Slip10Key(src)
    }
}

impl From<base64::DecodeError> for WalletDbError {
    fn from(src: base64::DecodeError) -> Self {
        Self::Base64Decode(src)
    }
}

impl From<mc_crypto_keys::KeyError> for WalletDbError {
    fn from(src: mc_crypto_keys::KeyError) -> Self {
        Self::KeyError(src)
    }
}

impl From<reqwest::Error> for WalletDbError {
    fn from(src: reqwest::Error) -> Self {
        Self::ReqwestError(src)
    }
}

impl From<ed25519_dalek::ed25519::Error> for WalletDbError {
    fn from(src: ed25519_dalek::ed25519::Error) -> Self {
        Self::Dalek(src)
    }
}

impl From<DecodeSliceError> for WalletDbError {
    fn from(src: DecodeSliceError) -> Self {
        Self::DecodeSlice(src)
    }
}
