// Copyright (c) 2020-2025 MobileCoin Inc.

//! Service for managing signed contingent inputs.

use displaydoc::Display;
use mc_connection::{BlockchainConnection, UserTxConnection};
use mc_fog_report_validation::FogPubkeyResolver;
use mc_ledger_db::{Error as LedgerError, Ledger};
use mc_transaction_extra::SignedContingentInput;
use mc_transaction_types::Amount;

use crate::{
    json_rpc::v2::models::signed_contingent_input::ValidateProofOfReserveSciResult, WalletService,
};

/// Errors for the SignedContingentInputService.
#[derive(Display, Debug)]
#[allow(clippy::large_enum_variant, clippy::result_large_err)]
pub enum SignedContingentInputServiceError {
    /// Error decoding prost: {0}
    ProstDecode(mc_util_serial::DecodeError),

    /// Error decoding from hex: {0}
    HexDecode(hex::FromHexError),

    /// Error from ledger: {0}
    LedgerError(LedgerError),
}

impl From<mc_util_serial::DecodeError> for SignedContingentInputServiceError {
    fn from(src: mc_util_serial::DecodeError) -> Self {
        Self::ProstDecode(src)
    }
}

impl From<hex::FromHexError> for SignedContingentInputServiceError {
    fn from(src: hex::FromHexError) -> Self {
        Self::HexDecode(src)
    }
}

impl From<LedgerError> for SignedContingentInputServiceError {
    fn from(src: LedgerError) -> Self {
        Self::LedgerError(src)
    }
}

/// Trait defining the ways in which the wallet can interact with and manage
/// signed contingent inputs.
#[rustfmt::skip]
#[allow(clippy::result_large_err)]
pub trait SignedContingentInputService {
    /// Validate a signed contingent input.
    ///
    /// # Arguments
    ///
    ///| Name               | Purpose                                                          | Notes                                                                             |
    ///|--------------------|------------------------------------------------------------------|-----------------------------------------------------------------------------------|
    ///| `sci_proto`        | The signed contingent input profobuf (hex-encoded)               |                                                                                   |
    ///
    fn validate_proof_of_reserve_sci(
        &self,
        sci_proto: &str,
    ) -> Result<ValidateProofOfReserveSciResult, SignedContingentInputServiceError>;
}

impl<T, FPR> SignedContingentInputService for WalletService<T, FPR>
where
    T: BlockchainConnection + UserTxConnection + 'static,
    FPR: FogPubkeyResolver + Send + Sync + 'static,
{
    fn validate_proof_of_reserve_sci(
        &self,
        sci_proto: &str,
    ) -> Result<ValidateProofOfReserveSciResult, SignedContingentInputServiceError> {
        let sci: SignedContingentInput = mc_util_serial::decode(&hex::decode(sci_proto)?)?;

        if let Err(err) = sci.validate() {
            return Ok(ValidateProofOfReserveSciResult::InvalidSci {
                error: err.to_string(),
            });
        }

        if sci.tx_in.ring.len() != 1 {
            return Ok(ValidateProofOfReserveSciResult::NotProofOfReserveSci {
                error: "Proof of reserve SCI must have exactly one ring member".to_string(),
            });
        }

        let ring_txo = &sci.tx_in.ring[0];
        let tx_out_public_key = hex::encode(ring_txo.public_key);
        let txo_index = match self
            .ledger_db
            .get_tx_out_index_by_public_key(&ring_txo.public_key)
        {
            Ok(txo_index) => txo_index,
            Err(LedgerError::NotFound) => {
                return Ok(ValidateProofOfReserveSciResult::TxOutNotFoundInLedger {
                    tx_out_public_key,
                });
            }
            Err(e) => return Err(SignedContingentInputServiceError::LedgerError(e)),
        };
        let ledger_txo = match self.ledger_db.get_tx_out_by_index(txo_index) {
            Ok(txo) => txo,
            Err(LedgerError::NotFound) => {
                return Ok(ValidateProofOfReserveSciResult::TxOutNotFoundInLedger {
                    tx_out_public_key,
                });
            }
            Err(e) => return Err(SignedContingentInputServiceError::LedgerError(e)),
        };
        if ring_txo != &ledger_txo {
            return Ok(ValidateProofOfReserveSciResult::TxOutMismatch { tx_out_public_key });
        }

        if sci.required_output_amounts.len() != 1 {
            return Ok(ValidateProofOfReserveSciResult::NotProofOfReserveSci {
                error: "Proof of reserve SCI must have exactly one required output".to_string(),
            });
        }

        let required_output_amount = &sci.required_output_amounts[0];
        if required_output_amount.token_id != sci.pseudo_output_amount.token_id {
            return Ok(ValidateProofOfReserveSciResult::NotProofOfReserveSci {
                error: "Proof of reserve SCI must have the same token id as the pseudo-output"
                    .to_string(),
            });
        }
        if required_output_amount.value != u64::MAX {
            return Ok(ValidateProofOfReserveSciResult::NotProofOfReserveSci {
                error: "Proof of reserve SCI must have the maximum value for the required output"
                    .to_string(),
            });
        }

        let Some(input_rules) = sci.tx_in.input_rules.as_ref() else {
            return Ok(ValidateProofOfReserveSciResult::NotProofOfReserveSci {
                error: "Proof of reserve SCI must have input rules".to_string(),
            });
        };
        if input_rules.max_tombstone_block != 1 {
            return Ok(ValidateProofOfReserveSciResult::NotProofOfReserveSci {
                error: "Proof of reserve SCI must have a max tombstone block of 1".to_string(),
            });
        }

        let amount = Amount::from(&sci.pseudo_output_amount);
        Ok(ValidateProofOfReserveSciResult::Valid {
            tx_out_public_key,
            key_image: hex::encode(&sci.key_image()),
            amount: (&amount).into(),
        })
    }
}
