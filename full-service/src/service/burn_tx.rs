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
        models::{Account, TransactionLog},
        transaction,
        transaction_log::{AssociatedTxos, TransactionLogModel, ValueMap},
        WalletDbError,
    },
    error::WalletTransactionBuilderError,
    json_rpc::v2::models::amount::Amount as AmountJSON,
    service::{
        ledger::LedgerService, models::tx_proposal::TxProposal,
        transaction_builder::WalletTransactionBuilder, WalletService,
    },
    util::b58::{b58_decode_public_address, B58Error},
};
use mc_account_keys::{burn_address, AccountKey};
use mc_common::logger::log;
use mc_connection::{BlockchainConnection, RetryableUserTxConnection, UserTxConnection};
use mc_fog_report_validation::FogPubkeyResolver;
use mc_transaction_core::{
    constants::{MAX_INPUTS, MAX_OUTPUTS},
    tokens::Mob,
    Amount, Token, TokenId,
};
use mc_transaction_std::{BurnRedemptionMemo, BurnRedemptionMemoBuilder, SenderMemoCredential};

use crate::{
    fog_resolver::FullServiceFogResolver,
    service::address::{AddressService, AddressServiceError},
    unsigned_tx::UnsignedTx,
};
use displaydoc::Display;
use std::convert::TryFrom;

/// Errors for the Transaction Service.
#[derive(Display, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum BurnTxServiceError {
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

impl From<std::num::ParseIntError> for BurnTxServiceError {
    fn from(_src: std::num::ParseIntError) -> Self {
        Self::U64Parse
    }
}

impl From<WalletTransactionBuilderError> for BurnTxServiceError {
    fn from(src: WalletTransactionBuilderError) -> Self {
        Self::TransactionBuilder(src)
    }
}

impl From<mc_api::ConversionError> for BurnTxServiceError {
    fn from(src: mc_api::ConversionError) -> Self {
        Self::ProtoConversion(src)
    }
}

impl From<retry::Error<mc_connection::Error>> for BurnTxServiceError {
    fn from(e: retry::Error<mc_connection::Error>) -> Self {
        Self::Connection(e)
    }
}

impl From<AddressServiceError> for BurnTxServiceError {
    fn from(e: AddressServiceError) -> Self {
        Self::AddressService(e)
    }
}

impl From<diesel::result::Error> for BurnTxServiceError {
    fn from(src: diesel::result::Error) -> Self {
        Self::Diesel(src)
    }
}

impl From<mc_ledger_db::Error> for BurnTxServiceError {
    fn from(src: mc_ledger_db::Error) -> Self {
        Self::LedgerDB(src)
    }
}
/// Trait defining the ways in which the wallet can interact with and manage
/// burn transactions.
pub trait BurnTxService {
    #[allow(clippy::too_many_arguments)]
    fn build_burn_transaction(
        &self,
        account_id_hex: &str,
        amount: &AmountJSON,
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
        amount: &AmountJSON,
        input_txo_ids: Option<&Vec<String>>,
        fee_value: Option<String>,
        fee_token_id: Option<String>,
        tombstone_block: Option<String>,
        max_spendable_value: Option<String>,
    ) -> Result<TxProposal, BurnTxServiceError> {
        let conn = self.wallet_db.get_conn()?;

        transaction(&conn, || {
            let account = Account::get(&AccountID(account_id_hex.to_string()), &conn)?;
            let from_account_key: AccountKey =
                mc_util_serial::decode(&account.account_key).unwrap();

            let mut builder = WalletTransactionBuilder::new(
                account_id_hex.to_string(),
                self.ledger_db.clone(),
                self.fog_resolver_factory.clone(),
                self.logger.clone(),
            );

            let amount = Amount::try_from(amount).map_err(BurnTxServiceError::InvalidAmount)?;
            let default_fee_token_id = amount.token_id;

            builder.add_recipient(burn_address(), amount.value, amount.token_id);

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
                None => *self
                    .get_network_fees()
                    .get(&fee_token_id)
                    .ok_or(BurnTxServiceError::DefaultFeeNotFoundForToken(fee_token_id))?,
            };

            builder.set_fee(fee_value, fee_token_id)?;

            builder.set_block_version(self.get_network_block_version());

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

            let memo_data = [1; BurnRedemptionMemo::MEMO_DATA_LEN];
            let mut memo_builder = BurnRedemptionMemoBuilder::new(memo_data);
            memo_builder.enable_destination_memo();
            let tx_proposal: TxProposal =
                builder.build(Some(Box::new(memo_builder)), &conn).unwrap();

            TransactionLog::log_built(tx_proposal.clone(), "".to_string(), account_id_hex, &conn)?;

            Ok(tx_proposal)
        })
    }
}

#[cfg(test)]
mod tests {}
