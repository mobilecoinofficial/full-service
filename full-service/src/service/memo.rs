use crate::{
    db::{
        account::{AccountID, AccountModel},
        models::Account,
        WalletDbError,
    },
    service::ledger::{LedgerService, LedgerServiceError},
    util::b58::{b58_decode_public_address, B58Error},
    WalletService,
};
use mc_account_keys::AccountKey;
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_crypto_keys::{KeyError, RistrettoPublic};
use mc_fog_report_validation::FogPubkeyResolver;
use mc_transaction_extra::{MemoDecodingError, MemoType, UnusedMemo};
use mc_util_serial::DecodeError;
use std::{convert::TryFrom, ops::DerefMut};
use thiserror::Error;

#[derive(Error, Debug)]
#[allow(clippy::large_enum_variant, clippy::result_large_err)]
pub enum MemoServiceError {
    #[error("WalletDb: {0}")]
    WalletDb(WalletDbError),

    #[error("B58: {0}")]
    B58(#[from] B58Error),

    #[error("Decode: {0}")]
    Decode(DecodeError),

    #[error("LedgerService: {0}")]
    LedgerService(LedgerServiceError),

    #[error("Key: {0}")]
    Key(KeyError),

    #[error("MemoDecoding: {0}")]
    MemoDecoding(MemoDecodingError),

    #[error("Invalid memo type for validation, expecting AuthenticatedSenderMemo.")]
    InvalidMemoTypeForValidation,
}

impl From<WalletDbError> for MemoServiceError {
    fn from(e: WalletDbError) -> Self {
        Self::WalletDb(e)
    }
}

impl From<DecodeError> for MemoServiceError {
    fn from(e: DecodeError) -> Self {
        Self::Decode(e)
    }
}

impl From<LedgerServiceError> for MemoServiceError {
    fn from(e: LedgerServiceError) -> Self {
        Self::LedgerService(e)
    }
}

impl From<KeyError> for MemoServiceError {
    fn from(e: KeyError) -> Self {
        Self::Key(e)
    }
}

impl From<MemoDecodingError> for MemoServiceError {
    fn from(e: MemoDecodingError) -> Self {
        Self::MemoDecoding(e)
    }
}

pub trait MemoService {
    fn validate_sender_memo(
        &self,
        account_id: &AccountID,
        txo_id_hex: &str,
        sender_address: &str,
    ) -> Result<bool, MemoServiceError>;
}

impl<T, FPR> MemoService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn validate_sender_memo(
        &self,
        account_id: &AccountID,
        txo_id_hex: &str,
        sender_address: &str,
    ) -> Result<bool, MemoServiceError> {
        let sender_address = b58_decode_public_address(sender_address)?;

        let mut pooled_conn = self.get_pooled_conn()?;
        let conn = pooled_conn.deref_mut();

        let account = Account::get(account_id, conn)?;
        let account_key: AccountKey = mc_util_serial::decode(&account.account_key)?;

        let txo = self.get_txo_object(txo_id_hex)?;
        let shared_secret =
            account.get_shared_secret(&RistrettoPublic::try_from(&txo.public_key)?)?;
        let memo_payload = match txo.e_memo {
            Some(e_memo) => e_memo.decrypt(&shared_secret),
            None => UnusedMemo.into(),
        };

        match MemoType::try_from(&memo_payload) {
            Ok(MemoType::AuthenticatedSender(memo)) => Ok(memo
                .validate(
                    &sender_address,
                    account_key.view_private_key(),
                    &txo.public_key,
                )
                .into()),
            Ok(_) => Err(MemoServiceError::InvalidMemoTypeForValidation),
            Err(e) => Err(e.into()),
        }
    }
}
