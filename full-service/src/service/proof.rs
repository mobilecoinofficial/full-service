// Copyright (c) 2020-2021 MobileCoin Inc.

//! Service for managing Proofs.

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
use mc_crypto_keys::CompressedRistrettoPublic;
use mc_fog_report_validation::FogPubkeyResolver;
use mc_ledger_db::Ledger;
use mc_transaction_core::tx::TxOutConfirmationNumber;

/// Errors for the Txo Service.
#[derive(Display, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum ProofServiceError {
    /// Error interacting with the database: {0}
    Database(WalletDbError),

    /// Diesel Error: {0}
    Diesel(diesel::result::Error),

    /// Error with LedgerDB: {0}
    LedgerDB(mc_ledger_db::Error),

    /// Error decoding prost: {0}
    ProstDecode(prost::DecodeError),

    /// Error decoding from hex: {0}
    HexDecode(hex::FromHexError),

    /// Minted Txo should contain proof: {0}
    MissingProof(String),

    /// Error with the TxoService: {0}
    TxoService(TxoServiceError),

    /// Error with the TxoService: {0}
    TransactionLogService(TransactionLogServiceError),
}

impl From<WalletDbError> for ProofServiceError {
    fn from(src: WalletDbError) -> Self {
        Self::Database(src)
    }
}

impl From<diesel::result::Error> for ProofServiceError {
    fn from(src: diesel::result::Error) -> Self {
        Self::Diesel(src)
    }
}

impl From<mc_ledger_db::Error> for ProofServiceError {
    fn from(src: mc_ledger_db::Error) -> Self {
        Self::LedgerDB(src)
    }
}

impl From<prost::DecodeError> for ProofServiceError {
    fn from(src: prost::DecodeError) -> Self {
        Self::ProstDecode(src)
    }
}

impl From<hex::FromHexError> for ProofServiceError {
    fn from(src: hex::FromHexError) -> Self {
        Self::HexDecode(src)
    }
}

impl From<TxoServiceError> for ProofServiceError {
    fn from(src: TxoServiceError) -> Self {
        Self::TxoService(src)
    }
}

impl From<TransactionLogServiceError> for ProofServiceError {
    fn from(src: TransactionLogServiceError) -> Self {
        Self::TransactionLogService(src)
    }
}

pub struct Proof {
    pub txo_id: TxoID,
    pub txo_index: u64,
    pub proof: TxOutConfirmationNumber,
}

/// Trait defining the ways in which the wallet can interact with and manage
/// Proofs.
pub trait ProofService {
    /// Get the proofs from the outputs in a transaction log.
    fn get_proofs(&self, transaction_log_id: &str) -> Result<Vec<Proof>, ProofServiceError>;

    /// Verify the proof with a given Txo.
    fn verify_proof(
        &self,
        account_id: &AccountID,
        txo_id: &TxoID,
        proof_hex: &str,
    ) -> Result<bool, ProofServiceError>;
}

impl<T, FPR> ProofService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn get_proofs(&self, transaction_log_id: &str) -> Result<Vec<Proof>, ProofServiceError> {
        let (_transaction_log, associated_txos) = self.get_transaction_log(&transaction_log_id)?;

        let mut results = Vec::new();
        for associated_txo in associated_txos.outputs {
            let txo = self.get_txo(&TxoID(associated_txo.clone()))?;
            if let Some(proof) = txo.txo.proof {
                let confirmation: TxOutConfirmationNumber = mc_util_serial::decode(&proof)?;
                let pubkey: CompressedRistrettoPublic =
                    mc_util_serial::decode(&txo.txo.public_key)?;
                let txo_index = self.ledger_db.get_tx_out_index_by_public_key(&pubkey)?;
                results.push(Proof {
                    txo_id: TxoID(txo.txo.txo_id_hex),
                    txo_index,
                    proof: confirmation,
                });
            } else {
                return Err(ProofServiceError::MissingProof(associated_txo));
            }
        }
        Ok(results)
    }

    fn verify_proof(
        &self,
        account_id: &AccountID,
        txo_id: &TxoID,
        proof_hex: &str,
    ) -> Result<bool, ProofServiceError> {
        let conn = self.wallet_db.get_conn()?;
        let proof: TxOutConfirmationNumber = mc_util_serial::decode(&hex::decode(proof_hex)?)?;
        Ok(Txo::verify_proof(
            &AccountID(account_id.to_string()),
            &txo_id.to_string(),
            &proof,
            &conn,
        )?)
    }
}
