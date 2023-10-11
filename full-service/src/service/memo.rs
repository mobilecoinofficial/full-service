use std::{
    ops::DerefMut,
    convert::TryFrom,
};
use crate::{
    db::{
        account::{AccountID, AccountModel},
        models::{Account, Txo},
        WalletDbError,
    },
    util::b58::{b58_decode_public_address, B58Error},
    service::ledger::{LedgerService, LedgerServiceError},
    WalletService,
};
use mc_account_keys::AccountKey;
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_fog_report_validation::FogPubkeyResolver;
use mc_crypto_keys::RistrettoPublic;
use mc_transaction_extra::{MemoType, UnusedMemo};
use displaydoc::Display;

#[derive(Display, Debug)]
#[allow(clippy::large_enum_variant, clippy::result_large_err)]
pub enum MemoServiceError {
    /// Error interacting with the database: {0}
    Database(WalletDbError),

    /// B58 Error
    B58(B58Error),

    /// Error decoding account key: {0}
    Decode(mc_util_serial::DecodeError),

    /// Error interacting with ledger database: {0}
    Ledger(LedgerServiceError),

    /// Key Error
    Key(mc_crypto_keys::KeyError),
}


impl From<WalletDbError> for MemoServiceError {
    fn from(src: WalletDbError) -> Self {
        Self::Database(src)
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
        Self::Ledger(src)
    }
}


impl From<mc_crypto_keys::KeyError> for MemoServiceError {
    fn from(src: mc_crypto_keys::KeyError) -> Self {
        Self::Key(src)
    }
}


#[rustfmt::skip]
#[allow(clippy::result_large_err)]
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
        Ok(
            if let Ok(memo_type) = MemoType::try_from(&memo_payload) {
                match memo_type {
                    MemoType::AuthenticatedSender(memo) => {
                        memo.validate(
                            &sender_address,
                            &account_key.view_private_key(),
                            &txo.public_key,
                        ).into()
                    }
                    _ => false,
                }
            } else {
                false
            }
        )
    }
}
