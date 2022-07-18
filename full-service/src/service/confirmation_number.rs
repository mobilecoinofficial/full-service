// Copyright (c) 2020-2021 MobileCoin Inc.

//! Service for managing confirmation numbers.

use std::convert::TryInto;

use crate::{
    db::{
        account::AccountID,
        models::Txo,
        txo::{TxoID, TxoModel},
        WalletDbError,
    },
    service::{
        transaction_log::{TransactionLogService, TransactionLogServiceError},
        txo::{TxoService, TxoServiceError},
    },
    WalletService,
};
use displaydoc::Display;
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_crypto_keys::{CompressedRistrettoPublic, GenericArray, ReprBytes, RistrettoPublic};
use mc_fog_report_validation::FogPubkeyResolver;
use mc_ledger_db::Ledger;
use mc_transaction_core::tx::TxOutConfirmationNumber;

/// Errors for the Txo Service.
#[derive(Display, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum ConfirmationServiceError {
    /// Error interacting with the database: {0}
    Database(WalletDbError),

    /// Diesel Error: {0}
    Diesel(diesel::result::Error),

    /// Error with LedgerDB: {0}
    LedgerDB(mc_ledger_db::Error),

    /// Error decoding prost: {0}
    ProstDecode(mc_util_serial::DecodeError),

    /// Error decoding from hex: {0}
    HexDecode(hex::FromHexError),

    /// Minted Txo should contain confirmation: {0}
    MissingConfirmation(String),

    /// Error with the TxoService: {0}
    TxoService(TxoServiceError),

    /// Error with the TxoService: {0}
    TransactionLogService(TransactionLogServiceError),

    /// TryFromSliceError: {0}
    TryFromSliceError(std::array::TryFromSliceError),

    /// Key Error: {0}
    KeyError(mc_crypto_keys::KeyError),

    /// From str Error: {0}
    FromStringError(String),
}

impl From<WalletDbError> for ConfirmationServiceError {
    fn from(src: WalletDbError) -> Self {
        Self::Database(src)
    }
}

impl From<diesel::result::Error> for ConfirmationServiceError {
    fn from(src: diesel::result::Error) -> Self {
        Self::Diesel(src)
    }
}

impl From<mc_ledger_db::Error> for ConfirmationServiceError {
    fn from(src: mc_ledger_db::Error) -> Self {
        Self::LedgerDB(src)
    }
}

impl From<mc_util_serial::DecodeError> for ConfirmationServiceError {
    fn from(src: mc_util_serial::DecodeError) -> Self {
        Self::ProstDecode(src)
    }
}

impl From<hex::FromHexError> for ConfirmationServiceError {
    fn from(src: hex::FromHexError) -> Self {
        Self::HexDecode(src)
    }
}

impl From<TxoServiceError> for ConfirmationServiceError {
    fn from(src: TxoServiceError) -> Self {
        Self::TxoService(src)
    }
}

impl From<TransactionLogServiceError> for ConfirmationServiceError {
    fn from(src: TransactionLogServiceError) -> Self {
        Self::TransactionLogService(src)
    }
}

impl From<std::array::TryFromSliceError> for ConfirmationServiceError {
    fn from(src: std::array::TryFromSliceError) -> Self {
        Self::TryFromSliceError(src)
    }
}

impl From<mc_crypto_keys::KeyError> for ConfirmationServiceError {
    fn from(src: mc_crypto_keys::KeyError) -> Self {
        Self::KeyError(src)
    }
}

impl From<&str> for ConfirmationServiceError {
    fn from(src: &str) -> Self {
        Self::FromStringError(src.to_string())
    }
}

#[derive(Debug)]
pub struct Confirmation {
    pub txo_id: TxoID,
    pub txo_index: u64,
    pub confirmation: TxOutConfirmationNumber,
}

/// Trait defining the ways in which the wallet can interact with and manage
/// tonfirmation numbers.
pub trait ConfirmationService {
    /// Get the confirmations from the outputs in a transaction log.
    fn get_confirmations(
        &self,
        transaction_log_id: &str,
    ) -> Result<Vec<Confirmation>, ConfirmationServiceError>;

    /// Validate the confirmation number with a given Txo.
    fn validate_confirmation(
        &self,
        account_id: &AccountID,
        txo_id: &TxoID,
        confirmation_hex: &str,
    ) -> Result<bool, ConfirmationServiceError>;
}

impl<T, FPR> ConfirmationService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn get_confirmations(
        &self,
        transaction_log_id: &str,
    ) -> Result<Vec<Confirmation>, ConfirmationServiceError> {
        let (_transaction_log, associated_txos, _value_map) =
            self.get_transaction_log(transaction_log_id)?;

        let mut results = Vec::new();
        for (associated_txo, _) in associated_txos.outputs {
            let (txo, _) = self.get_txo(&TxoID(associated_txo.id.clone()))?;
            if let Some(shared_secret_bytes) = txo.shared_secret {
                let shared_secret_bytes: [u8; 32] = shared_secret_bytes.as_slice().try_into()?;
                let shared_secret =
                    RistrettoPublic::from_bytes(GenericArray::from_slice(&shared_secret_bytes))?;

                let confirmation = TxOutConfirmationNumber::from(&shared_secret);
                let pubkey: CompressedRistrettoPublic = mc_util_serial::decode(&txo.public_key)?;
                let txo_index = self.ledger_db.get_tx_out_index_by_public_key(&pubkey)?;
                results.push(Confirmation {
                    txo_id: TxoID(txo.id),
                    txo_index,
                    confirmation,
                });
            } else {
                return Err(ConfirmationServiceError::MissingConfirmation(
                    associated_txo.id,
                ));
            }
        }
        Ok(results)
    }

    fn validate_confirmation(
        &self,
        account_id: &AccountID,
        txo_id: &TxoID,
        confirmation_hex: &str,
    ) -> Result<bool, ConfirmationServiceError> {
        let conn = self.wallet_db.get_conn()?;
        let confirmation_bytes: [u8; 32] = hex::decode(confirmation_hex)?.as_slice().try_into()?;
        let confirmation =
            TxOutConfirmationNumber::from_bytes(GenericArray::from_slice(&confirmation_bytes))?;

        Ok(Txo::validate_confirmation(
            &AccountID(account_id.to_string()),
            &txo_id.to_string(),
            &confirmation,
            &conn,
        )?)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        db::account::AccountID,
        service::{
            account::AccountService, address::AddressService,
            confirmation_number::ConfirmationService, transaction::TransactionService,
        },
        test_utils::{
            add_block_from_transaction_log, add_block_to_ledger_db, get_test_ledger,
            manually_sync_account, setup_wallet_service, MOB,
        },
    };
    use mc_account_keys::{AccountKey, PublicAddress};
    use mc_common::logger::{test_with_logger, Logger};
    use mc_crypto_keys::ReprBytes;
    use mc_crypto_rand::rand_core::RngCore;
    use mc_transaction_core::{ring_signature::KeyImage, tokens::Mob, Token};
    use rand::{rngs::StdRng, SeedableRng};

    #[test_with_logger]
    fn test_validate_confirmation(logger: Logger) {
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
                "".to_string(),
            )
            .unwrap();

        let alice_account_key: AccountKey = mc_util_serial::decode(&alice.account_key).unwrap();
        let alice_account_id = AccountID::from(&alice_account_key);
        let alice_public_address = alice_account_key.subaddress(alice.main_subaddress_index as u64);

        add_block_to_ledger_db(
            &mut ledger_db,
            &vec![alice_public_address.clone()],
            100 * MOB,
            &vec![KeyImage::from(rng.next_u64())],
            &mut rng,
        );

        manually_sync_account(&ledger_db, &service.wallet_db, &alice_account_id, &logger);

        let address = service
            .assign_address_for_account(&alice_account_id, None)
            .unwrap();

        let (transaction_log, _, _, _) = service
            .build_and_submit(
                &alice_account_id.to_string(),
                &[(
                    address.assigned_subaddress_b58.clone(),
                    (50 * MOB).to_string(),
                    Mob::ID.to_string(),
                )],
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .unwrap();

        {
            let conn = service.wallet_db.get_conn().unwrap();
            add_block_from_transaction_log(&mut ledger_db, &conn, &transaction_log, &mut rng);
        }

        manually_sync_account(&ledger_db, &service.wallet_db, &alice_account_id, &logger);

        let confirmations = service.get_confirmations(&transaction_log.id).unwrap();
        assert_eq!(&confirmations.len(), &1);
        let confirmation = &confirmations[0];
        let confirmation_hex =
            hex::encode(confirmation.confirmation.to_bytes().to_vec().as_slice());
        let validated = service
            .validate_confirmation(&alice_account_id, &confirmation.txo_id, &confirmation_hex)
            .unwrap();
        assert!(validated);
    }
}
