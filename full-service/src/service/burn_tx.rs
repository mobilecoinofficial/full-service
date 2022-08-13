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
        transaction, WalletDbError,
    },
    json_rpc::v2::models::amount::Amount as AmountJSON,
    service::{
        account::AccountServiceError,
        address::{AddressService, AddressServiceError},
        models::tx_proposal::TxProposal,
        transaction::{TransactionService, TransactionServiceError},
        transaction_builder::DEFAULT_NEW_TX_BLOCK_ATTEMPTS,
        WalletService,
    },
    util::b58::{
        b58_decode_public_address, b58_decode_transfer_payload, b58_encode_public_address,
        b58_encode_transfer_payload, B58Error, DecodedTransferPayload,
    },
};
use bip39::{Language, Mnemonic, MnemonicType};
use displaydoc::Display;
use mc_account_keys::{AccountKey, DEFAULT_SUBADDRESS_INDEX};
use mc_account_keys_slip10::Slip10KeyGenerator;
use mc_common::{logger::log, HashSet};
use mc_connection::{BlockchainConnection, RetryableUserTxConnection, UserTxConnection};
use mc_crypto_keys::RistrettoPublic;
use mc_crypto_ring_signature_signer::NoKeysRingSigner;
use mc_fog_report_validation::FogPubkeyResolver;
use mc_ledger_db::Ledger;
use mc_transaction_core::{
    constants::RING_SIZE,
    get_tx_out_shared_secret,
    onetime_keys::recover_onetime_private_key,
    ring_signature::KeyImage,
    tokens::Mob,
    tx::{Tx, TxOut},
    Amount, BlockVersion, Token,
};
use mc_transaction_std::{
    InputCredentials, RTHMemoBuilder, SenderMemoCredential, TransactionBuilder,
};
use mc_util_uri::FogUri;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt, iter::empty, str::FromStr, sync::atomic::Ordering};

#[derive(Display, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum BurnTxServiceError {
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

    /// Amount Error: {0}
    Amount(mc_transaction_core::AmountError),
}

impl From<WalletDbError> for BurnTxServiceError {
    fn from(src: WalletDbError) -> Self {
        Self::Database(src)
    }
}

impl From<B58Error> for BurnTxServiceError {
    fn from(src: B58Error) -> Self {
        Self::B58(src)
    }
}

impl From<mc_ledger_db::Error> for BurnTxServiceError {
    fn from(src: mc_ledger_db::Error) -> Self {
        Self::LedgerDB(src)
    }
}

impl From<hex::FromHexError> for BurnTxServiceError {
    fn from(src: hex::FromHexError) -> Self {
        Self::HexDecode(src)
    }
}

impl From<mc_util_serial::DecodeError> for BurnTxServiceError {
    fn from(src: mc_util_serial::DecodeError) -> Self {
        Self::ProstDecode(src)
    }
}

impl From<diesel::result::Error> for BurnTxServiceError {
    fn from(src: diesel::result::Error) -> Self {
        Self::Diesel(src)
    }
}

impl From<TransactionServiceError> for BurnTxServiceError {
    fn from(src: TransactionServiceError) -> Self {
        Self::TransactionService(src)
    }
}

impl From<AccountServiceError> for BurnTxServiceError {
    fn from(src: AccountServiceError) -> Self {
        Self::AccountService(src)
    }
}

impl From<mc_api::display::Error> for BurnTxServiceError {
    fn from(src: mc_api::display::Error) -> Self {
        Self::PrintableWrapper(src)
    }
}

impl From<mc_crypto_keys::KeyError> for BurnTxServiceError {
    fn from(src: mc_crypto_keys::KeyError) -> Self {
        Self::CryptoKey(src)
    }
}

impl From<mc_transaction_std::TxBuilderError> for BurnTxServiceError {
    fn from(src: mc_transaction_std::TxBuilderError) -> Self {
        Self::TxBuilder(src)
    }
}

impl From<mc_api::ConversionError> for BurnTxServiceError {
    fn from(src: mc_api::ConversionError) -> Self {
        Self::ProtoConversion(src)
    }
}

impl From<mc_util_uri::UriParseError> for BurnTxServiceError {
    fn from(src: mc_util_uri::UriParseError) -> Self {
        Self::UriParse(src)
    }
}

impl From<retry::Error<mc_connection::Error>> for BurnTxServiceError {
    fn from(e: retry::Error<mc_connection::Error>) -> Self {
        Self::Connection(e)
    }
}

impl From<AddressServiceError> for BurnTxServiceError {
    fn from(src: AddressServiceError) -> Self {
        Self::AddressService(src)
    }
}

impl From<mc_transaction_core::AmountError> for BurnTxServiceError {
    fn from(src: mc_transaction_core::AmountError) -> Self {
        Self::Amount(src)
    }
}

/// Trait defining the ways in which the wallet can interact with and manage
/// burn transactions.
pub trait BurnTxService {
    #[allow(clippy::too_many_arguments)]
    fn build_burn_transaction(
        &self,
        account_id_hex: &str,
        amount: AmountJSON,
        input_txo_ids: Option<&Vec<String>>,
        fee_value: Option<String>,
        fee_token_id: Option<String>,
        tombstone_block: Option<String>,
        max_spendable_value: Option<String>,
    ) -> Result<TxProposal, BurnTxServiceError>;
}

impl<T, FPR> BurnTxService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn build_burn_transaction(
        &self,
        account_id_hex: &str,
        amount: AmountJSON,
        input_txo_ids: Option<&Vec<String>>,
        fee_value: Option<String>,
        fee_token_id: Option<String>,
        tombstone_block: Option<String>,
        max_spendable_value: Option<String>,
    ) -> Result<TxProposal, BurnTxServiceError> {
        unimplemented!();
        // let conn = self.wallet_db.get_conn()?;
        // let from_account = Account::get(from_account_id, &conn)?;

        // let fee_value = fee.map(|f| f.to_string());

        // let tx_proposal = self.build_transaction(
        //     &from_account.id,
        //     &[(
        //         gift_code_account_main_subaddress_b58,
        //         crate::json_rpc::v2::models::amount::Amount {
        //             value: value.to_string(),
        //             token_id: Mob::ID.to_string(),
        //         },
        //     )],
        //     input_txo_ids,
        //     fee_value,
        //     None,
        //     tombstone_block.map(|t| t.to_string()),
        //     max_spendable_value.map(|f| f.to_string()),
        //     None,
        // )?;

        // if tx_proposal.payload_txos.len() != 1 {
        //     return Err(BurnTxServiceError::UnexpectedTxProposalFormat);
        // }

        // let tx_out = &tx_proposal.payload_txos[0].tx_out;

        // let proto_tx_pubkey: mc_api::external::CompressedRistretto =
        // (&tx_out.public_key).into();

        // let gift_code_b58 = b58_encode_transfer_payload(
        //     gift_code_bip39_entropy_bytes.to_vec(),
        //     proto_tx_pubkey,
        //     memo.unwrap_or_else(|| "".to_string()),
        // )?;

        // Ok((tx_proposal, EncodedGiftCode(gift_code_b58)))
    }
}

#[cfg(test)]
mod tests {}
