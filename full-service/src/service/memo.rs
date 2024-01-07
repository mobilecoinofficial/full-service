use crate::{
    db::{account::AccountModel, models::Txo, txo::TxoModel, WalletDbError},
    service::ledger::{LedgerService, LedgerServiceError},
    util::b58::{b58_decode_public_address, B58Error},
    WalletService,
};
use displaydoc::Display;
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_crypto_keys::RistrettoPublic;
use mc_fog_report_validation::FogPubkeyResolver;
use mc_transaction_extra::{MemoDecodingError, MemoType, UnusedMemo};
use std::{convert::TryFrom, ops::DerefMut};

#[derive(Display, Debug)]
#[allow(clippy::large_enum_variant, clippy::result_large_err)]
pub enum MemoServiceError {
    /// WalletDb: {0}
    WalletDb(WalletDbError),

    /// B58: {0}
    B58(B58Error),

    /// Decode: {0}
    Decode(mc_util_serial::DecodeError),

    ///LedgerService: {0}
    LedgerService(LedgerServiceError),

    /// Key: {0}
    Key(mc_crypto_keys::KeyError),

    /// MemoDecoding: {0}
    MemoDecoding(MemoDecodingError),

    /// Invalid memo type for validation, expecting AuthenticatedSenderMemo.
    InvalidMemoTypeForValidation,

    /// Unknown subaddress index for txo_id {0}. Can't validate.
    TxoOrphaned(String),
}

impl From<WalletDbError> for MemoServiceError {
    fn from(src: WalletDbError) -> Self {
        Self::WalletDb(src)
    }
}

impl From<B58Error> for MemoServiceError {
    fn from(src: B58Error) -> Self {
        Self::B58(src)
    }
}

impl From<mc_util_serial::DecodeError> for MemoServiceError {
    fn from(src: mc_util_serial::DecodeError) -> Self {
        Self::Decode(src)
    }
}

impl From<LedgerServiceError> for MemoServiceError {
    fn from(src: LedgerServiceError) -> Self {
        Self::LedgerService(src)
    }
}

impl From<mc_crypto_keys::KeyError> for MemoServiceError {
    fn from(src: mc_crypto_keys::KeyError) -> Self {
        Self::Key(src)
    }
}

impl From<MemoDecodingError> for MemoServiceError {
    fn from(src: MemoDecodingError) -> Self {
        Self::MemoDecoding(src)
    }
}

pub trait MemoService {
    fn validate_sender_memo(
        &self,
        txo_id_hex: &str,
        sender_address: &str,
    ) -> Result<bool, MemoServiceError>;
}

// validate_sender_memo
//
// validating the sender memo includes:
// 1. Is there a sender memo?
// 2. Does the provided sender public address hash to the
//    same value as the memo's sender address hash?
// 3. When we recreate the HMAC, does it match the
//    HMAC conveyed in the memo?
impl<T, FPR> MemoService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn validate_sender_memo(
        &self,
        txo_id_hex: &str,
        sender_address: &str,
    ) -> Result<bool, MemoServiceError> {
        let sender_address = b58_decode_public_address(sender_address)?;

        let mut pooled_conn = self.get_pooled_conn()?;
        let conn = pooled_conn.deref_mut();

        let txo = Txo::get(txo_id_hex, conn)?;
        let Some(account) = txo.account(conn)? else {
            return Ok(false);
        };

        // validating the HMAC requires the receipient's subaddress
        // view private key, so fetch the recipient subaddress_index
        // of the txo, and fail if this is not available (orphaned txo)
        let subaddress_index = match txo.subaddress_index {
            Some(subaddress_index) => subaddress_index as u64,
            // None => return Ok(false),
            None => return Err(MemoServiceError::TxoOrphaned(txo_id_hex.to_string())),
        };

        let account_key = account.account_key()?;

        let tx_out = self.get_txo_object(txo_id_hex)?;
        let shared_secret =
            account.get_shared_secret(&RistrettoPublic::try_from(&tx_out.public_key)?)?;
        let memo_payload = match tx_out.e_memo {
            Some(e_memo) => e_memo.decrypt(&shared_secret),
            None => UnusedMemo.into(),
        };

        match MemoType::try_from(&memo_payload) {
            Ok(MemoType::AuthenticatedSender(memo)) => Ok(memo
                .validate(
                    &sender_address,
                    &account_key.subaddress_view_private(subaddress_index),
                    &tx_out.public_key,
                )
                .into()),
            Ok(_) => Err(MemoServiceError::InvalidMemoTypeForValidation),
            Err(e) => Err(e.into()),
        }
    }
}
